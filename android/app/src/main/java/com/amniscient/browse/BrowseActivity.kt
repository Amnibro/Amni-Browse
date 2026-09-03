package com.amniscient.browse
import android.app.DownloadManager
import android.app.PictureInPictureParams
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.content.pm.ActivityInfo
import android.content.res.Configuration
import android.animation.ObjectAnimator
import android.animation.ValueAnimator
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.GradientDrawable
import android.view.animation.LinearInterpolator
import androidx.core.graphics.ColorUtils
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.Settings
import android.util.Rational
import android.util.TypedValue
import android.text.Editable
import android.text.TextWatcher
import android.webkit.CookieManager
import android.webkit.URLUtil
import android.widget.ImageView
import android.content.ClipData
import android.content.ClipboardManager
import android.print.PrintAttributes
import android.print.PrintManager
import android.view.ContextMenu
import android.view.DragEvent
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.ListPopupWindow
import androidx.documentfile.provider.DocumentFile
import androidx.swiperefreshlayout.widget.SwipeRefreshLayout
import androidx.webkit.WebSettingsCompat
import androidx.webkit.WebViewFeature
import org.json.JSONObject
import java.net.URLEncoder
import android.view.Gravity
import android.view.View
import android.view.autofill.AutofillManager
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.ValueCallback
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.EditText
import android.widget.ImageButton
import android.widget.LinearLayout
import android.widget.ListView
import android.widget.PopupMenu
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.content.res.AppCompatResources
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL
class BrowseActivity : AppCompatActivity() {
    private lateinit var web: WebView
    private lateinit var urlBar: EditText
    private lateinit var navRow: View
    private lateinit var btnBack: ImageButton
    private lateinit var btnForward: ImageButton
    private lateinit var btnRefresh: ImageButton
    private lateinit var loadPulse: LoadPulseBar
    private lateinit var progress: ProgressBar
    private lateinit var homePanel: View
    private lateinit var missingWeb: View
    private lateinit var retryBar: View
    private lateinit var tabStrip: LinearLayout
    private lateinit var bookmarkList: ListView
    private lateinit var db: AppDb
    private val tabs = mutableListOf(Tab("", "Home", true))
    private var tabIndex = 0
    private var customView: View? = null
    private var customCb: WebChromeClient.CustomViewCallback? = null
    private val navigationState = MainFrameNavigationState()
    private var fileChooserCallback: ValueCallback<Array<Uri>>? = null
    private var desktopSite = false
    private var mobileUA: String? = null
    private var loadingTabIndex = -1
    private var loadingIconUrl: String? = null
    private var iconEpoch = 0
    private val iconCache = mutableMapOf<String, Bitmap>()
    private var refreshSpin: ObjectAnimator? = null
    private lateinit var tabStripScroll: android.widget.HorizontalScrollView
    private lateinit var tabStripWrap: FrameLayout
    private lateinit var tabFadeLeft: View
    private lateinit var tabFadeRight: View
    private lateinit var chromeShell: ChromeShellLayout
    private lateinit var swipe: SwipeRefreshLayout
    private lateinit var bmBar: LinearLayout
    private lateinit var bmBarWrap: View
    private var suggestPop: ListPopupWindow? = null
    private var suggestRows: List<Pair<String, String>> = emptyList()
    private var suggestSeq = 0
    private var suggestJob: Job? = null
    private var saveJob: Job? = null
    private var paintJob: Job? = null
    private val ui by lazy { getSharedPreferences("amni_ui", Context.MODE_PRIVATE) }
    private fun jsBlocked(): MutableSet<String> = (ui.getStringSet("jsblock", emptySet()) ?: emptySet()).toMutableSet()
    private fun cookieBlocked(): MutableSet<String> = (ui.getStringSet("cookblock", emptySet()) ?: emptySet()).toMutableSet()
    private fun engineId(): String = ui.getString("engine", "ddg") ?: "ddg"
    private enum class TabSize(val stripDp: Float, val favDp: Float, val textSp: Float, val titleLen: Int, val closeSp: Float, val plusSp: Float, val chipPadH: Float, val chipPadV: Float) {
        COMPACT(40f, 14f, 12.5f, 20, 11f, 22f, 10f, 3f),
        DEFAULT(52f, 18f, 14f, 26, 12f, 28f, 12f, 5f),
        LARGE(64f, 22f, 16f, 32, 13f, 34f, 14f, 6f),
    }
    private fun tabSize(): TabSize = when (ui.getString("tabsize", "default")) {
        "compact" -> TabSize.COMPACT
        "large" -> TabSize.LARGE
        else -> TabSize.DEFAULT
    }
    private fun pullRefreshEnabled(): Boolean = ui.getBoolean("pullrefresh", false)
    private fun applyTabStripHeight() {
        val h = dp(tabSize().stripDp)
        tabStripWrap.layoutParams = tabStripWrap.layoutParams.also { it.height = h }
    }
    private fun applyPullRefresh() {
        swipe.isEnabled = pullRefreshEnabled()
        if (!swipe.isEnabled) swipe.isRefreshing = false
    }
    private fun selectedTheme(): Int = when (ui.getString("theme", "scient")) {
        "emerald" -> R.style.Theme_AmniBrowse_Emerald
        "haven" -> R.style.Theme_AmniBrowse_Haven
        "learn" -> R.style.Theme_AmniBrowse_Learn
        "amni" -> R.style.Theme_AmniBrowse_Amni
        "crypt" -> R.style.Theme_AmniBrowse_Crypt
        "ai" -> R.style.Theme_AmniBrowse_Ai
        "core" -> R.style.Theme_AmniBrowse_Core
        "explore" -> R.style.Theme_AmniBrowse_Explore
        "calc" -> R.style.Theme_AmniBrowse_Calc
        "braid" -> R.style.Theme_AmniBrowse_Braid
        "light" -> R.style.Theme_AmniBrowse_Light
        else -> R.style.Theme_AmniBrowse
    }
    private fun themeColor(attr: Int): Int {
        val value = TypedValue()
        theme.resolveAttribute(attr, value, true)
        return value.data
    }
    private lateinit var webMain: WebView
    private var webPriv: WebView? = null
    private val pageClient = object : WebViewClient() {
        override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
            if (view !== web) return
            navigationState.onPageStarted(url)
            renderNavigationFailure()
            renderOmniboxDecor(url)
            beginTabIconLoad(tabIndex, url)
            setPageLoading(true)
            web.settings.javaScriptEnabled = hostOf(url ?: "") !in jsBlocked()
            if (tabs.getOrNull(tabIndex)?.priv == true) web.settings.saveFormData = false
            applyCookieGate(url ?: "")
        }
        override fun onPageFinished(view: WebView?, url: String?) {
            if (view !== web) return
            swipe.isRefreshing = false
            val verifiedSuccess = navigationState.onPageFinished(url)
            if (verifiedSuccess) retryBar.visibility = View.GONE
            else             renderNavigationFailure()
            renderOmniboxDecor(url)
            updateNavButtons()
            if (!verifiedSuccess) {
                setPageLoading(false)
                return
            }
            if (url.isNullOrBlank() || url.startsWith("about:")) {
                setPageLoading(false)
                return
            }
            tabs[tabIndex].url = url
            tabs[tabIndex].title = view?.title ?: tabs[tabIndex].title
            tabs[tabIndex].home = false
            if (!urlBar.hasFocus()) urlBar.setText(displayUrl(url))
            schedulePaintTabs()
            saveSession()
            if (tabs.getOrNull(tabIndex)?.priv != true) recordHistory(url, view?.title)
        }
        override fun onReceivedError(view: WebView?, request: WebResourceRequest?, error: WebResourceError?) {
            if (view !== web) return
            if (request?.isForMainFrame == true) {
                navigationState.onMainFrameError(request.url?.toString() ?: tabs[tabIndex].url)
                renderNavigationFailure()
            }
        }
        override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
            val u = request?.url ?: return false
            val raw = u.toString()
            if (BrowserPolicies.isDangerousScheme(raw)) return true
            val sch = u.scheme ?: return false
            if (sch == "http" || sch == "https") {
                val s = NavResolver.stripTrackers(raw)
                applyCookieGate(s)
                if (s != raw) { view?.loadUrl(s); return true }
                return false
            }
            if (BrowserPolicies.canOpenExternally(raw)) {
                openExternalUrl(view, raw)
                return true
            }
            // Unknown / non-http schemes: do not load in-app and do not hand off blindly.
            return true
        }
    }
    private val chromeClient = object : WebChromeClient() {
        override fun onProgressChanged(view: WebView?, newProgress: Int) {
            if (view !== web) return
            progress.progress = newProgress
            val loading = newProgress in 1..99
            loadPulse.accentColor = themeColor(R.attr.amniAccent)
            loadPulse.progress = newProgress / 100f
            loadPulse.active = loading
            loadPulse.visibility = if (loading) View.VISIBLE else View.GONE
            setRefreshSpinning(loading)
        }
        override fun onReceivedTitle(view: WebView?, title: String?) {
            if (view !== web) return
            tabs[tabIndex].title = title ?: tabs[tabIndex].title
            schedulePaintTabs()
        }
        override fun onReceivedIcon(view: WebView?, icon: Bitmap?) {
            if (view !== web || icon == null) return
            val idx = tabIndex
            if (idx !in tabs.indices) return
            tabs[idx].icon = icon
            hostOf(view.url ?: tabs[idx].url).takeIf { it.isNotBlank() }?.let { iconCache[it] = icon }
            renderOmniboxDecor(view.url ?: tabs[idx].url)
            schedulePaintTabs()
        }
        override fun onShowFileChooser(
            webView: WebView?,
            filePathCallback: ValueCallback<Array<Uri>>?,
            fileChooserParams: WebChromeClient.FileChooserParams?,
        ): Boolean {
            fileChooserCallback?.onReceiveValue(null)
            fileChooserCallback = filePathCallback ?: return false
            return try {
                val intent = fileChooserParams?.createIntent()
                    ?: Intent(Intent.ACTION_OPEN_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE)
                val accepted = BrowserPolicies.fileChooserAcceptTypes(fileChooserParams?.acceptTypes)
                if (accepted.size == 1) intent.type = accepted[0]
                else {
                    intent.type = "*/*"
                    intent.putExtra(Intent.EXTRA_MIME_TYPES, accepted)
                }
                chooseWebFile.launch(intent)
                true
            } catch (_: ActivityNotFoundException) {
                fileChooserCallback?.onReceiveValue(null)
                fileChooserCallback = null
                Toast.makeText(this@BrowseActivity, "No file picker is available", Toast.LENGTH_LONG).show()
                true
            }
        }
        override fun onShowCustomView(view: View?, callback: WebChromeClient.CustomViewCallback?) {
            if (view == null) return
            if (customView != null) { callback?.onCustomViewHidden(); return }
            customView = view
            customCb = callback
            (window.decorView as ViewGroup).addView(view, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
            requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
        }
        override fun onHideCustomView() { hideCustomView() }
    }
    private fun setupWebView(w: WebView) {
        w.settings.javaScriptEnabled = true
        w.settings.domStorageEnabled = true
        w.settings.mixedContentMode = android.webkit.WebSettings.MIXED_CONTENT_NEVER_ALLOW
        w.settings.mediaPlaybackRequiresUserGesture = true
        w.settings.setGeolocationEnabled(false)
        w.settings.allowFileAccess = false
        w.settings.allowContentAccess = false
        @Suppress("DEPRECATION")
        run {
            w.settings.allowFileAccessFromFileURLs = false
            w.settings.allowUniversalAccessFromFileURLs = false
        }
        w.settings.textZoom = ui.getInt("textzoom", 100)
        w.importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_YES
        CookieManager.getInstance().setAcceptThirdPartyCookies(w, false)
        if (WebViewFeature.isFeatureSupported(WebViewFeature.SAFE_BROWSING_ENABLE)) {
            WebSettingsCompat.setSafeBrowsingEnabled(w.settings, true)
        }
        if (mobileUA == null) mobileUA = w.settings.userAgentString
        if (ui.getBoolean("darkpages", false) && WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING))
            WebSettingsCompat.setAlgorithmicDarkeningAllowed(w.settings, true)
        w.webViewClient = pageClient
        w.webChromeClient = chromeClient
        registerForContextMenu(w)
        w.setDownloadListener { url, userAgent, contentDisposition, mimetype, _ ->
            if (!BrowserPolicies.canDownload(url)) {
                Toast.makeText(this, "This download link is not supported", Toast.LENGTH_LONG).show()
                return@setDownloadListener
            }
            try {
                val req = DownloadManager.Request(Uri.parse(url))
                req.setMimeType(mimetype)
                req.addRequestHeader("User-Agent", userAgent)
                CookieManager.getInstance().getCookie(url)?.let { req.addRequestHeader("Cookie", it) }
                val name = URLUtil.guessFileName(url, contentDisposition, mimetype)
                req.setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                req.setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, name)
                (getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager).enqueue(req)
                Toast.makeText(this, "Downloading $name", Toast.LENGTH_SHORT).show()
            } catch (e: Exception) { Toast.makeText(this, e.message ?: "download failed", Toast.LENGTH_LONG).show() }
        }
    }
    /** Kill HTML media so closed/switched tabs cannot keep playing as ghosts. */
    private fun silenceWeb(w: WebView = web) {
        try { w.evaluateJavascript(BrowserPolicies.SILENCE_JS, null) } catch (_: Exception) {}
        try { w.stopLoading() } catch (_: Exception) {}
    }
    private fun trashPrivWeb() {
        val w = webPriv ?: return
        silenceWeb(w)
        // Destroy the instance — do not park about:blank (that keeps a document alive).
        try { w.clearCache(true); w.clearFormData(); w.clearHistory() } catch (_: Exception) {}
        try { (w.parent as? ViewGroup)?.removeView(w) } catch (_: Exception) {}
        try { w.destroy() } catch (_: Exception) {}
        webPriv = null
        if (WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
            try { androidx.webkit.ProfileStore.getInstance().getProfile("private")?.cookieManager?.removeAllCookies(null) } catch (_: Exception) {}
        }
    }
    private fun ensurePrivWeb(): WebView {
        webPriv?.let { return it }
        val w = WebView(this)
        // A real profile: its own cookies, storage and cache — this is the isolation the
        // shared-CookieManager private tab could not give.
        if (WebViewFeature.isFeatureSupported(WebViewFeature.MULTI_PROFILE)) {
            val store = androidx.webkit.ProfileStore.getInstance()
            store.getOrCreateProfile("private")
            androidx.webkit.WebViewCompat.setProfile(w, "private")
        }
        setupWebView(w)
        w.settings.saveFormData = false
        w.visibility = View.GONE
        findViewById<android.widget.FrameLayout>(R.id.webwrap).addView(w, android.widget.FrameLayout.LayoutParams(-1, -1))
        webPriv = w
        return w
    }
    private fun switchWeb(priv: Boolean) {
        val target = if (priv) ensurePrivWeb() else webMain
        if (target === web) return
        web.visibility = View.GONE
        web = target
        web.visibility = View.VISIBLE
    }
    private val chooseWebFile = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        val selected = WebChromeClient.FileChooserParams.parseResult(result.resultCode, result.data)
        fileChooserCallback?.onReceiveValue(selected)
        fileChooserCallback = null
    }
    private val openDoc = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) importUri(uri)
    }
    private val openTree = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            getSharedPreferences("amni_autoimport", Context.MODE_PRIVATE).edit().putString("tree", uri.toString()).apply()
            Toast.makeText(this, "Watching this folder — new browser exports import themselves", Toast.LENGTH_LONG).show()
            scanAutoImport()
        }
    }
    private fun dp(v: Float): Int = (v * resources.displayMetrics.density).toInt()
    override fun onCreate(savedInstanceState: Bundle?) {
        setTheme(selectedTheme())
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_browse)
        // Modern target SDKs draw edge-to-edge; without this the tab strip
        // lives under the status bar and the toolbar under the gesture pill.
        val root = findViewById<View>(R.id.root)
        androidx.core.view.ViewCompat.setOnApplyWindowInsetsListener(root) { v, insets ->
            val bars = insets.getInsets(androidx.core.view.WindowInsetsCompat.Type.systemBars())
            v.setPadding(0, bars.top, 0, bars.bottom)
            insets
        }
        db = AppDb.get(this)
        webMain = findViewById(R.id.web)
        web = webMain
        urlBar = findViewById(R.id.urlBar)
        progress = findViewById(R.id.progress)
        homePanel = findViewById(R.id.homePanel)
        missingWeb = findViewById(R.id.missingWeb)
        retryBar = findViewById(R.id.retryBar)
        navRow = findViewById(R.id.navRow)
        btnBack = findViewById(R.id.btnBack)
        btnForward = findViewById(R.id.btnForward)
        btnRefresh = findViewById(R.id.btnRefresh)
        loadPulse = findViewById(R.id.loadPulse)
        loadPulse.accentColor = themeColor(R.attr.amniAccent)
        tabStrip = findViewById(R.id.tabStrip)
        tabStripScroll = findViewById(R.id.tabStripScroll)
        tabStripWrap = findViewById(R.id.tabStripWrap)
        tabFadeLeft = findViewById(R.id.tabFadeLeft)
        tabFadeRight = findViewById(R.id.tabFadeRight)
        chromeShell = findViewById(R.id.chromeShell)
        bookmarkList = findViewById(R.id.bookmarkList)
        swipe = findViewById(R.id.swipe)
        bmBar = findViewById(R.id.bmBar)
        bmBarWrap = findViewById(R.id.bmBarWrap)
        applyTabStripHeight()
        applyPullRefresh()
        setupTabEdgeFades()
        swipe.setColorSchemeColors(themeColor(R.attr.amniAccent))
        swipe.setProgressBackgroundColorSchemeColor(themeColor(R.attr.amniBgTertiary))
        swipe.setOnRefreshListener {
            performControl(BrowserControl.RELOAD)
            if (homePanel.visibility == View.VISIBLE) swipe.isRefreshing = false
        }
        swipe.setOnChildScrollUpCallback { _, _ ->
            if (!pullRefreshEnabled()) return@setOnChildScrollUpCallback true
            web.scrollY > 0
        }
        if (ui.getBoolean("bmbar", false)) { bmBarWrap.visibility = View.VISIBLE; paintBmBar() }
        if (WebView.getCurrentWebViewPackage() == null) {
            missingWeb.visibility = View.VISIBLE
            homePanel.visibility = View.GONE
            findViewById<Button>(R.id.btnFixWebview).setOnClickListener {
                try {
                    startActivity(Intent(Intent.ACTION_VIEW, Uri.parse("market://details?id=com.google.android.webview")))
                } catch (_: ActivityNotFoundException) {
                    startActivity(Intent(Settings.ACTION_SETTINGS))
                }
            }
        }
        setupWebView(webMain)
        getSystemService(AutofillManager::class.java)?.notifyViewEntered(web)
        btnBack.setOnClickListener { performControl(BrowserControl.BACK) }
        btnForward.setOnClickListener { performControl(BrowserControl.FORWARD) }
        btnRefresh.setOnClickListener { performControl(BrowserControl.RELOAD) }
        findViewById<ImageButton>(R.id.btnHome).setOnClickListener { showHome() }
        findViewById<ImageButton>(R.id.btnTabs).setOnClickListener { tabGrid() }
        findViewById<ImageButton>(R.id.btnMenu).setOnClickListener { showMenu(it) }
        findViewById<Button>(R.id.btnImport).setOnClickListener { openDoc.launch(arrayOf("application/json", "text/plain", "*/*")) }
        findViewById<Button>(R.id.btnDefault).setOnClickListener { openDefaultSettings() }
        findViewById<Button>(R.id.btnRetry).setOnClickListener { navigationState.failedUrl?.let { go(it) } }
        urlBar.setOnEditorActionListener { _, action, _ ->
            if (action == EditorInfo.IME_ACTION_GO || action == EditorInfo.IME_ACTION_DONE) {
                suggestPop?.dismiss(); go(urlBar.text.toString()); true
            } else false
        }
        urlBar.addTextChangedListener(object : TextWatcher {
            override fun afterTextChanged(s: Editable?) {
                val q = s?.toString()?.trim() ?: ""
                if (!urlBar.hasFocus() || q.length < 2) { suggestPop?.dismiss(); suggestJob?.cancel(); return }
                val seq = ++suggestSeq
                suggestJob?.cancel()
                suggestJob = lifecycleScope.launch {
                    delay(250)
                    if (seq != suggestSeq) return@launch
                    val hist = withContext(Dispatchers.IO) { db.dao().searchHistory(q, 4) }
                    val bms = withContext(Dispatchers.IO) { db.dao().searchBookmarks(q, 3) }
                    if (seq != suggestSeq) return@launch
                    val rows = LinkedHashMap<String, String>()
                    for (b in bms) rows[b.url] = "★ " + b.title.ifBlank { b.url }
                    for (h in hist) if (!rows.containsKey(h.url)) rows[h.url] = (h.title.ifBlank { h.url })
                    val net = withContext(Dispatchers.IO) { fetchSuggest(q) }
                    for (s in net) if (!rows.containsValue(s) && s !in rows.keys) rows["search:$s"] = s
                    suggestRows = rows.map { it.key to it.value }
                    if (suggestRows.isEmpty()) { suggestPop?.dismiss(); return@launch }
                    val pop = suggestPop ?: ListPopupWindow(this@BrowseActivity).also { suggestPop = it; it.anchorView = urlBar; it.isModal = false }
                    pop.setAdapter(ArrayAdapter(this@BrowseActivity, android.R.layout.simple_list_item_1, suggestRows.map { it.second + "\n" + it.first }))
                    pop.setOnItemClickListener { _, _, i, _ ->
                        suggestPop?.dismiss()
                        val key = suggestRows[i].first
                        go(if (key.startsWith("search:")) key.removePrefix("search:") else key)
                    }
                    pop.show()
                }
            }
            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) {}
            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) {}
        })
        urlBar.setOnFocusChangeListener { _, has ->
            if (!has) suggestPop?.dismiss()
        }
        updateNavButtons()
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                if (customView != null) { hideCustomView(); return }
                if (web.visibility == View.VISIBLE && web.canGoBack()) web.goBack() else finish()
            }
        })
        SessionStore.load(this)?.let { (saved, idx) ->
            tabs.clear()
            tabs.addAll(saved)
            tabIndex = idx
            normalizeGroups()
        }
        savedInstanceState?.getString(STATE_TABS)?.let { raw ->
            SessionCodec.decodeOrNull(raw, savedInstanceState.getInt(STATE_INDEX, 0))?.let { (saved, idx) ->
                if (saved.isNotEmpty()) {
                    tabs.clear()
                    tabs.addAll(saved)
                    tabIndex = idx
                    normalizeGroups()
                }
            }
        }
        handleIntent(intent)
        paintTabs()
        refreshBookmarks()
        val t = tabs[tabIndex]
        if (t.home || t.url.isEmpty()) showHome() else go(t.url)
    }
    override fun onStop() {
        saveJob?.cancel()
        saveSessionNow()
        super.onStop()
    }
    override fun onSaveInstanceState(outState: Bundle) {
        val enc = SessionCodec.encode(tabs, tabIndex)
        outState.putString(STATE_TABS, enc.first)
        outState.putInt(STATE_INDEX, enc.second)
        super.onSaveInstanceState(outState)
    }
    override fun onDestroy() {
        refreshSpin?.cancel()
        fileChooserCallback?.onReceiveValue(null)
        fileChooserCallback = null
        super.onDestroy()
    }
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }
    private fun saveSessionNow() = SessionStore.save(this, tabs, tabIndex)
    private fun saveSession() {
        saveJob?.cancel()
        saveJob = lifecycleScope.launch {
            delay(400)
            saveSessionNow()
        }
    }
    private fun schedulePaintTabs() {
        paintJob?.cancel()
        paintJob = lifecycleScope.launch {
            delay(80)
            paintTabs()
        }
    }
    private fun handleIntent(intent: Intent?) {
        intent ?: return
        // Share an export from any browser straight into AmniBrowse and it imports.
        if (intent.action == Intent.ACTION_SEND) {
            @Suppress("DEPRECATION") val stream = intent.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
            if (stream != null) { importUri(stream); return }
            val txt = intent.getStringExtra(Intent.EXTRA_TEXT)
            if (!txt.isNullOrBlank()) { if (txt.trimStart().startsWith("http")) go(txt.trim()) else importText(txt, "shared text"); return }
        }
        val data = intent.dataString
        if (intent.action == Intent.ACTION_VIEW && !data.isNullOrBlank()) {
            if (data.startsWith("content:") || data.startsWith("file:")) importUri(Uri.parse(data)) else go(data)
        }
    }
    private fun hostOf(url: String): String = try { Uri.parse(url).host ?: url } catch (_: Exception) { url }
    private fun renderNavigationFailure() {
        retryBar.visibility = if (navigationState.hasFailure()) View.VISIBLE else View.GONE
        navigationState.failedUrl?.let { failed ->
            findViewById<TextView>(R.id.retryText).text = getString(R.string.page_failed_for, hostOf(failed))
        }
    }
    private fun renderOmniboxDecor(url: String?) {
        val insecure = BrowserPolicies.connectionSecurity(url) == ConnectionSecurity.INSECURE_HTTP
        when {
            insecure -> {
                val icon = AppCompatResources.getDrawable(this, R.drawable.ic_insecure)?.mutate()
                icon?.setTint(themeColor(R.attr.amniDanger))
                urlBar.setCompoundDrawablesWithIntrinsicBounds(icon, null, null, null)
            }
            tabs.getOrNull(tabIndex)?.icon != null && !tabs[tabIndex].home -> {
                val bmp = tabs[tabIndex].icon!!
                val d = BitmapDrawable(resources, bmp)
                val size = dp(18f)
                d.setBounds(0, 0, size, size)
                urlBar.setCompoundDrawables(d, null, null, null)
            }
            else -> urlBar.setCompoundDrawablesWithIntrinsicBounds(null, null, null, null)
        }
        urlBar.compoundDrawablePadding = dp(8f)
        urlBar.contentDescription = when {
            insecure -> getString(R.string.omnibox_insecure_description)
            else -> getString(R.string.omnibox_hint)
        }
    }
    private fun updateNavButtons() {
        val onPage = homePanel.visibility != View.VISIBLE && web.visibility == View.VISIBLE
        val backOk = onPage && web.canGoBack()
        val fwdOk = onPage && web.canGoForward()
        btnBack.visibility = View.VISIBLE
        btnForward.visibility = View.VISIBLE
        btnBack.alpha = if (backOk) 1f else 0.38f
        btnForward.alpha = if (fwdOk) 1f else 0.38f
        btnRefresh.alpha = if (onPage) 1f else 0.38f
        btnBack.isEnabled = true
        btnForward.isEnabled = true
        btnRefresh.isEnabled = onPage
    }
    private fun setRefreshSpinning(spin: Boolean) {
        if (spin) {
            if (refreshSpin?.isRunning == true) return
            refreshSpin = ObjectAnimator.ofFloat(btnRefresh, View.ROTATION, 0f, 360f).apply {
                duration = 900L
                repeatCount = ValueAnimator.INFINITE
                interpolator = LinearInterpolator()
                start()
            }
        } else {
            refreshSpin?.cancel()
            refreshSpin = null
            btnRefresh.rotation = 0f
        }
    }
    private fun performControl(control: BrowserControl) {
        val onPage = homePanel.visibility != View.VISIBLE && web.visibility == View.VISIBLE
        when (control) {
            BrowserControl.BACK -> when {
                onPage && web.canGoBack() -> {
                    web.goBack()
                    web.post { updateNavButtons() }
                }
                onPage -> showHome()
            }
            BrowserControl.FORWARD -> if (onPage && web.canGoForward()) {
                web.goForward()
                web.post { updateNavButtons() }
            }
            BrowserControl.RELOAD -> if (onPage) {
                if (navigationState.hasFailure()) {
                    navigationState.failedUrl?.let { go(it) } ?: web.reload()
                } else {
                    web.reload()
                }
            }
        }
    }
    private fun openExternalUrl(source: WebView?, url: String) {
        if (!BrowserPolicies.canOpenExternally(url)) {
            Toast.makeText(this, "Blocked unsupported link", Toast.LENGTH_SHORT).show()
            return
        }
        try {
            if (url.startsWith("intent:", ignoreCase = true)) {
                val external = Intent.parseUri(url, Intent.URI_INTENT_SCHEME)
                try {
                    startActivity(external)
                } catch (_: ActivityNotFoundException) {
                    val fallback = external.getStringExtra("browser_fallback_url")
                    if (BrowserPolicies.connectionSecurity(fallback) in setOf(ConnectionSecurity.SECURE, ConnectionSecurity.INSECURE_HTTP)) {
                        source?.loadUrl(fallback!!)
                    } else {
                        Toast.makeText(this, "No app can open this link", Toast.LENGTH_SHORT).show()
                    }
                }
            } else {
                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
            }
        } catch (_: Exception) {
            Toast.makeText(this, "No app can open this link", Toast.LENGTH_SHORT).show()
        }
    }
    private fun displayUrl(url: String?): String {
        val u = url ?: return ""
        if (u.startsWith("about:") || u.startsWith("data:")) return ""
        return u
    }
    private fun dismissOmnibox() {
        suggestPop?.dismiss()
        urlBar.clearFocus()
        getSystemService(InputMethodManager::class.java)?.hideSoftInputFromWindow(urlBar.windowToken, 0)
    }
    private fun go(raw: String) {
        switchWeb(tabs[tabIndex].priv)
        val url = NavResolver.resolve(raw, engineId())
        navigationState.onPageStarted(url)
        renderNavigationFailure()
        renderOmniboxDecor(url)
        tabs[tabIndex].home = false
        tabs[tabIndex].url = url
        setPageLoading(true)
        homePanel.visibility = View.GONE
        web.visibility = View.VISIBLE
        applyCookieGate(url)
        web.loadUrl(url)
        urlBar.setText(url)
        dismissOmnibox()
        paintTabs()
        saveSession()
    }
    private fun showHome() {
        switchWeb(tabs[tabIndex].priv)
        tabs[tabIndex].home = true
        tabs[tabIndex].title = "Home"
        tabs[tabIndex].url = ""
        iconEpoch++
        loadingTabIndex = tabIndex
        loadingIconUrl = null
        tabs[tabIndex].icon = null
        silenceWeb()
        homePanel.visibility = View.VISIBLE
        web.visibility = View.GONE
        renderNavigationFailure()
        renderOmniboxDecor(null)
        updateNavButtons()
        urlBar.setText("")
        dismissOmnibox()
        refreshBookmarks()
        paintTabs()
        saveSession()
    }
    private fun newTab() {
        snapThumb()
        tabs.add(Tab("", "Home", true, priv = false))
        tabIndex = tabs.lastIndex
        showHome()
    }
    private fun newOpenTab() {
        snapThumb()
        tabs.add(Tab("", "Home", true, priv = false))
        tabIndex = tabs.lastIndex
        showHome()
        Toast.makeText(this, "Open tab — this one is saved", Toast.LENGTH_SHORT).show()
    }
    private fun closeTab() {
        if (tabs.size == 1) { showHome(); return }
        val wasPriv = tabs[tabIndex].priv
        silenceWeb() // drop media first
        tabs.removeAt(tabIndex) // then remove tab
        if (wasPriv && tabs.none { it.priv }) trashPrivWeb()
        tabIndex = tabIndex.coerceAtMost(tabs.lastIndex)
        val t = tabs[tabIndex]
        if (t.home || t.url.isEmpty()) showHome() else go(t.url)
    }
    private fun setPageLoading(loading: Boolean) {
        loadPulse.accentColor = themeColor(R.attr.amniAccent)
        loadPulse.active = loading
        loadPulse.visibility = if (loading) View.VISIBLE else View.GONE
        if (!loading) loadPulse.progress = 0f
        setRefreshSpinning(loading)
    }
    private fun beginTabIconLoad(tabIdx: Int, url: String?) {
        val epoch = ++iconEpoch
        loadingTabIndex = tabIdx
        loadingIconUrl = url
        val tab = tabs.getOrNull(tabIdx) ?: return
        val host = hostOf(url ?: "")
        if (host.isBlank() || url.isNullOrBlank() || !url.startsWith("http")) {
            if (tab.home) tab.icon = null
            schedulePaintTabs()
            return
        }
        iconCache[host]?.let { cached ->
            tab.icon = cached
            renderOmniboxDecor(url)
            schedulePaintTabs()
            return
        }
        val priorHost = hostOf(tab.url)
        if (priorHost.isNotBlank() && priorHost != host) tab.icon = null
        schedulePaintTabs()
        lifecycleScope.launch {
            delay(500)
            if (epoch != iconEpoch) return@launch
            if (tabs.getOrNull(tabIdx)?.icon != null) return@launch
            val bmp = withContext(Dispatchers.IO) { downloadFavicon(host) }
            if (epoch != iconEpoch || bmp == null) return@launch
            if (tabs.getOrNull(tabIdx)?.icon != null) return@launch
            if (hostOf(loadingIconUrl ?: "") != host && hostOf(tabs.getOrNull(tabIdx)?.url ?: "") != host) return@launch
            tabs[tabIdx].icon = bmp
            iconCache[host] = bmp
            renderOmniboxDecor(tabs[tabIdx].url)
            schedulePaintTabs()
        }
    }
    private fun downloadFavicon(host: String): Bitmap? {
        val candidates = listOf(
            "https://$host/favicon.ico",
            "https://$host/favicon.png",
            "https://$host/apple-touch-icon.png",
        )
        for (u in candidates) {
            try {
                val c = java.net.URL(u).openConnection() as java.net.HttpURLConnection
                c.connectTimeout = 2500
                c.readTimeout = 2500
                c.instanceFollowRedirects = true
                c.setRequestProperty("User-Agent", "Mozilla/5.0")
                val code = c.responseCode
                if (code !in 200..299) { c.disconnect(); continue }
                val bytes = c.inputStream.use { it.readBytes() }
                c.disconnect()
                if (bytes.size < 16 || bytes.size > 400_000) continue
                val raw = BitmapFactory.decodeByteArray(bytes, 0, bytes.size) ?: continue
                return if (raw.width > 64 || raw.height > 64)
                    Bitmap.createScaledBitmap(raw, 64, 64, true).also { if (it !== raw) raw.recycle() }
                else raw
            } catch (_: Exception) { /* try next */ }
        }
        return null
    }
    private fun groupAccent(name: String): Int {
        val hue = ((name.hashCode().toLong() and 0x7fffffffL) % 360L).toFloat()
        return Color.HSVToColor(floatArrayOf(hue, 0.55f, 0.82f))
    }
    private fun clusterGroup(name: String, focusIdx: Int) {
        if (focusIdx !in tabs.indices) return
        val active = tabs.getOrNull(tabIndex)
        val focus = tabs[focusIdx]
        val members = tabs.filterIndexed { i, t -> t.group == name && i != focusIdx }.toMutableList()
        members.add(0, focus)
        val others = tabs.filter { it.group != name }.toMutableList()
        val insertAt = tabs.take(focusIdx).count { it.group != name }.coerceIn(0, others.size)
        others.addAll(insertAt, members)
        tabs.clear()
        tabs.addAll(others)
        tabIndex = active?.let { tabs.indexOf(it) }?.takeIf { it >= 0 } ?: tabs.indexOf(focus).coerceAtLeast(0)
    }
    private fun assignGroup(tabIdx: Int, name: String?) {
        if (tabIdx !in tabs.indices) return
        tabs[tabIdx].group = name
        if (!name.isNullOrBlank()) clusterGroup(name, tabIdx)
        paintTabs()
        saveSession()
    }
    private fun normalizeGroups() {
        val seen = linkedSetOf<String>()
        tabs.mapNotNull { it.group?.takeIf { g -> g.isNotBlank() } }.forEach { seen.add(it) }
        for (name in seen) {
            val first = tabs.indexOfFirst { it.group == name }
            if (first >= 0) clusterGroup(name, first)
        }
    }
    private enum class ChipRole { ACTIVE, NEAR, FAR }
    private fun chipRole(i: Int): ChipRole {
        val d = kotlin.math.abs(i - tabIndex)
        return when {
            i == tabIndex -> ChipRole.ACTIVE
            d == 1 -> ChipRole.NEAR
            d == 2 && tabs.size <= 7 -> ChipRole.NEAR
            else -> ChipRole.FAR
        }
    }
    private data class ChipStyle(
        val titleLen: Int,
        val textSp: Float,
        val favDp: Float,
        val padH: Float,
        val showTitle: Boolean,
        val showClose: Boolean,
        val maxWidthDp: Float,
        val textAlpha: Float,
    )
    private fun chipStyle(role: ChipRole, base: TabSize): ChipStyle {
        val crowded = tabs.size >= 8
        val packed = tabs.size >= 12
        return when (role) {
            ChipRole.ACTIVE -> ChipStyle(
                titleLen = base.titleLen,
                textSp = base.textSp,
                favDp = base.favDp,
                padH = base.chipPadH,
                showTitle = true,
                showClose = true,
                maxWidthDp = if (crowded) 200f else 240f,
                textAlpha = 1f,
            )
            ChipRole.NEAR -> ChipStyle(
                titleLen = (base.titleLen * 0.55f).toInt().coerceIn(8, 16),
                textSp = (base.textSp - 0.5f).coerceAtLeast(11f),
                favDp = (base.favDp - 2f).coerceAtLeast(14f),
                padH = (base.chipPadH - 2f).coerceAtLeast(6f),
                showTitle = true,
                showClose = !crowded,
                maxWidthDp = if (crowded) 112f else 140f,
                textAlpha = 0.88f,
            )
            ChipRole.FAR -> ChipStyle(
                titleLen = if (packed) 0 else if (crowded) 4 else 8,
                textSp = (base.textSp - 1.5f).coerceAtLeast(10f),
                favDp = (base.favDp - 4f).coerceAtLeast(12f),
                padH = if (packed || crowded) 6f else 8f,
                showTitle = !packed && !crowded,
                showClose = false,
                maxWidthDp = when {
                    packed -> 44f
                    crowded -> 72f
                    else -> 96f
                },
                textAlpha = 0.72f,
            )
        }
    }
    private fun setupTabEdgeFades() {
        val bg = themeColor(R.attr.amniBgSecondary)
        tabFadeLeft.background = GradientDrawable(
            GradientDrawable.Orientation.LEFT_RIGHT,
            intArrayOf(bg, Color.TRANSPARENT),
        )
        tabFadeRight.background = GradientDrawable(
            GradientDrawable.Orientation.RIGHT_LEFT,
            intArrayOf(bg, Color.TRANSPARENT),
        )
        tabStripScroll.setOnScrollChangeListener { _, _, _, _, _ ->
            updateTabEdgeFades()
            updateChromeOutline()
        }
    }
    private fun updateTabEdgeFades() {
        val child = tabStripScroll.getChildAt(0) ?: return
        val canLeft = tabStripScroll.scrollX > 2
        val canRight = tabStripScroll.scrollX + tabStripScroll.width < child.width - 2
        tabFadeLeft.animate().cancel()
        tabFadeRight.animate().cancel()
        tabFadeLeft.alpha = if (canLeft) 1f else 0f
        tabFadeRight.alpha = if (canRight) 1f else 0f
        tabFadeLeft.visibility = if (canLeft) View.VISIBLE else View.INVISIBLE
        tabFadeRight.visibility = if (canRight) View.VISIBLE else View.INVISIBLE
    }
    private fun findTabChip(idx: Int): View? {
        fun walk(group: ViewGroup): View? {
            for (c in 0 until group.childCount) {
                val ch = group.getChildAt(c)
                if (ch.tag == idx) return ch
                if (ch is ViewGroup) walk(ch)?.let { return it }
            }
            return null
        }
        return walk(tabStrip)
    }
    private fun offsetInStrip(v: View): Int {
        var x = v.left
        var p = v.parent
        while (p is View && p !== tabStrip) {
            x += p.left
            p = p.parent
        }
        return x
    }
    private fun ensureActiveTabVisible() {
        tabStrip.post {
            val chip = findTabChip(tabIndex) ?: return@post
            val left = offsetInStrip(chip)
            val right = left + chip.width
            val pad = dp(36f)
            val sl = tabStripScroll.scrollX
            val sr = sl + tabStripScroll.width
            val target = when {
                left - pad < sl -> (left - pad).coerceAtLeast(0)
                right + pad > sr -> (right + pad - tabStripScroll.width).coerceAtLeast(0)
                else -> sl
            }
            if (target != sl) tabStripScroll.smoothScrollTo(target, 0)
            else updateTabEdgeFades()
            tabStrip.post { updateTabEdgeFades() }
            updateChromeOutline()
        }
    }
    private fun updateChromeOutline() {
        chromeShell.post {
            val chip = findTabChip(tabIndex)
            if (chip == null) {
                chromeShell.outlineVisible = false
                return@post
            }
            val shellLoc = IntArray(2)
            val chipLoc = IntArray(2)
            chromeShell.getLocationOnScreen(shellLoc)
            chip.getLocationOnScreen(chipLoc)
            chromeShell.tabLeft = (chipLoc[0] - shellLoc[0]).toFloat()
            chromeShell.tabRight = chromeShell.tabLeft + chip.width
            chromeShell.tabTop = (chipLoc[1] - shellLoc[1]).toFloat()
            chromeShell.bodyTop = tabStripWrap.height.toFloat()
            chromeShell.borderColor = themeColor(R.attr.amniBorder)
            chromeShell.strokePx = resources.displayMetrics.density
            chromeShell.cornerPx = dp(10f).toFloat()
            chromeShell.outlineVisible = true
        }
    }
    private fun paintTabChip(i: Int, t: Tab, base: TabSize, inGroup: Boolean): LinearLayout {
        val role = chipRole(i)
        val st = chipStyle(role, base)
        val active = i == tabIndex
        val chip = LinearLayout(this)
        chip.orientation = LinearLayout.HORIZONTAL
        chip.gravity = Gravity.CENTER_VERTICAL
        chip.setBackgroundResource(if (active) R.drawable.tab_active else R.drawable.tab_inactive)
        val padBottom = if (active) 0f else base.chipPadV * 0.35f
        chip.setPadding(dp(st.padH), dp(base.chipPadV), dp(if (st.showClose) 3f else st.padH), dp(padBottom))
        val lp = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.MATCH_PARENT)
        lp.marginEnd = if (inGroup) dp(2f) else dp(4f)
        if (role == ChipRole.NEAR) lp.marginEnd = dp(5f)
        if (!active) lp.bottomMargin = dp(2f)
        chip.layoutParams = lp
        chip.minimumWidth = dp(if (role == ChipRole.FAR && !st.showTitle) 40f else 48f)
        chip.alpha = 1f
        val fav = t.icon
        if (fav != null && !t.home) {
            val iv = ImageView(this)
            iv.setImageBitmap(fav)
            iv.layoutParams = LinearLayout.LayoutParams(dp(st.favDp), dp(st.favDp)).also {
                it.marginEnd = if (st.showTitle) dp(5f) else 0
            }
            chip.addView(iv)
        } else if (role == ChipRole.FAR && !st.showTitle) {
            val letter = TextView(this)
            val label = t.title.ifBlank { if (t.home) "H" else "?" }
            letter.text = label.take(1).uppercase()
            letter.textSize = st.textSp
            letter.setTextColor(themeColor(R.attr.amniTextSecondary))
            letter.gravity = Gravity.CENTER
            letter.layoutParams = LinearLayout.LayoutParams(dp(st.favDp), dp(st.favDp))
            chip.addView(letter)
        }
        if (st.showTitle) {
            val tv = TextView(this)
            val raw = (if (t.priv) "🕶 " else "") + (if (t.title.isBlank()) "New tab" else t.title)
            tv.text = if (st.titleLen <= 0) "" else raw.take(st.titleLen)
            tv.setTextColor(if (active) themeColor(R.attr.amniTextPrimary) else themeColor(R.attr.amniTextSecondary))
            tv.textSize = st.textSp
            tv.alpha = st.textAlpha
            tv.maxWidth = dp(st.maxWidthDp - st.favDp - 28f)
            tv.isSingleLine = true
            tv.ellipsize = android.text.TextUtils.TruncateAt.END
            chip.addView(tv)
        }
        if (st.showClose) {
            val x = TextView(this)
            x.text = "✕"
            x.textSize = base.closeSp
            x.setTextColor(themeColor(R.attr.amniTextSecondary))
            x.setPadding(dp(7f), dp(3f), dp(6f), dp(3f))
            x.setOnClickListener { closeTabAt(i) }
            chip.addView(x)
        }
        chip.setOnClickListener { switchTab(i) }
        chip.tag = i
        chip.setOnLongClickListener {
            showTabGroupMenu(i, chip)
            true
        }
        chip.setOnDragListener { _, ev ->
            when (ev.action) {
                DragEvent.ACTION_DROP -> {
                    val from = ev.localState as? Int ?: return@setOnDragListener true
                    tabIndex = TabOrder.move(tabs, from, i, tabIndex)
                    normalizeGroups()
                    paintTabs(); saveSession()
                }
            }
            true
        }
        return chip
    }
    private fun paintTabs() {
        tabStrip.removeAllViews()
        val sz = tabSize()
        var i = 0
        while (i < tabs.size) {
            val g = tabs[i].group?.takeIf { it.isNotBlank() }
            if (g != null) {
                var end = i + 1
                while (end < tabs.size && tabs[end].group == g) end++
                val accent = groupAccent(g)
                val cluster = LinearLayout(this)
                cluster.orientation = LinearLayout.HORIZONTAL
                cluster.gravity = Gravity.CENTER_VERTICAL
                cluster.background = GradientDrawable().apply {
                    cornerRadius = dp(12f).toFloat()
                    setColor(ColorUtils.setAlphaComponent(accent, 0x28))
                    setStroke(dp(1.5f), ColorUtils.setAlphaComponent(accent, 0xCC))
                }
                cluster.setPadding(dp(4f), dp(2f), dp(4f), dp(2f))
                val cLp = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.MATCH_PARENT)
                cLp.marginEnd = dp(8f)
                cLp.marginStart = if (i > 0) dp(4f) else 0
                cluster.layoutParams = cLp
                val header = TextView(this)
                header.text = g
                header.textSize = (sz.textSp - 1.5f).coerceAtLeast(10f)
                header.setTextColor(Color.WHITE)
                header.setPadding(dp(8f), dp(3f), dp(8f), dp(3f))
                header.background = GradientDrawable().apply {
                    cornerRadius = dp(999f).toFloat()
                    setColor(accent)
                }
                header.gravity = Gravity.CENTER
                val hLp = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT)
                hLp.gravity = Gravity.CENTER_VERTICAL
                hLp.marginEnd = dp(4f)
                header.layoutParams = hLp
                val groupStart = i
                header.setOnClickListener { switchTab(groupStart) }
                header.setOnLongClickListener {
                    showTabGroupMenu(groupStart, header)
                    true
                }
                cluster.addView(header)
                for (idx in i until end) {
                    cluster.addView(paintTabChip(idx, tabs[idx], sz, inGroup = true))
                }
                tabStrip.addView(cluster)
                i = end
            } else {
                tabStrip.addView(paintTabChip(i, tabs[i], sz, inGroup = false))
                i++
            }
        }
        val plus = TextView(this)
        plus.text = "+"
        plus.textSize = sz.plusSp
        plus.setTextColor(themeColor(R.attr.amniAccent))
        plus.setPadding(dp(12f), dp(2f), dp(14f), dp(4f))
        plus.setOnClickListener { newTab() }
        plus.gravity = Gravity.CENTER
        tabStrip.addView(plus)
        ensureActiveTabVisible()
    }
    private fun closeTabAt(i: Int) {
        if (tabs.size == 1) { showHome(); return }
        val closingCurrent = i == tabIndex
        val wasPriv = tabs[i].priv
        if (closingCurrent) silenceWeb() // drop media first
        tabs.removeAt(i) // then remove tab
        if (wasPriv && tabs.none { it.priv }) trashPrivWeb()
        if (tabIndex > i || tabIndex > tabs.lastIndex) tabIndex--
        tabIndex = tabIndex.coerceIn(0, tabs.lastIndex)
        if (closingCurrent) {
            val t = tabs[tabIndex]
            if (t.home || t.url.isEmpty()) showHome() else go(t.url)
        } else { paintTabs(); saveSession() }
    }
    private fun showMenu(anchor: View) {
        val p = PopupMenu(this, anchor)
        p.menu.add("New tab")
        p.menu.add("New open tab")
        p.menu.add("New private tab")
        p.menu.add("Close tab")
        p.menu.add("All tabs…")
        val pg = p.menu.addSubMenu("Page")
        pg.add("Home"); pg.add("Forward")
        pg.add("Bookmark this page"); pg.add("Find in page"); pg.add("Share page")
        pg.add(if (desktopSite) "Mobile site" else "Desktop site")
        pg.add("Translate"); pg.add("Reader mode"); pg.add("Print / Save as PDF")
        pg.add("Picture-in-picture")
        pg.add(if (hostOf(tabs[tabIndex].url) in jsBlocked()) "Allow JavaScript on this site" else "Block JavaScript on this site")
        pg.add(if (hostOf(tabs[tabIndex].url) in cookieBlocked()) "Allow cookies on this site" else "Block cookies on this site")
        val lib = p.menu.addSubMenu("Library")
        lib.add("History"); lib.add("Bookmarks"); lib.add(if (ui.getBoolean("bmbar", false)) "Hide bookmarks bar" else "Show bookmarks bar")
        lib.add("Import browser file"); lib.add("Auto-import folder…"); lib.add("Clear browsing data")
        val st = p.menu.addSubMenu("Settings")
        st.add("Theme"); st.add("Tab size"); st.add("Pull to refresh"); st.add("Search engine"); st.add("Text size"); st.add(if (ui.getBoolean("darkpages", false)) "Dark pages: on" else "Dark pages: off"); st.add("Set as default")
        p.setOnMenuItemClickListener {
            when (it.title.toString()) {
                "New tab" -> newTab()
                "New open tab" -> newOpenTab()
                "New private tab" -> newPrivateTab()
                "Close tab" -> closeTab()
                "All tabs…" -> tabGrid()
                "Home" -> showHome()
                "Forward" -> if (web.canGoForward()) web.goForward()
                "Bookmark this page" -> addBookmark()
                "Find in page" -> findInPage()
                "Share page" -> sharePage()
                "Desktop site", "Mobile site" -> toggleDesktop()
                "Translate" -> translatePage()
                "Reader mode" -> readerMode()
                "Print / Save as PDF" -> printPage()
                "Allow JavaScript on this site", "Block JavaScript on this site" -> toggleJsBlock()
                "Allow cookies on this site", "Block cookies on this site" -> toggleCookieBlock()
                "Picture-in-picture" -> enterPip()
                "Search engine" -> searchEngineDialog()
                "Theme" -> themeDialog()
                "Tab size" -> tabSizeDialog()
                "Pull to refresh" -> togglePullRefresh()
                "History" -> showHistory()
                "Bookmarks" -> bookmarksManager()
                "Show bookmarks bar", "Hide bookmarks bar" -> toggleBmBar()
                "Import browser file" -> openDoc.launch(arrayOf("application/json", "text/html", "text/plain", "*/*"))
                "Auto-import folder…" -> openTree.launch(null)
                "Clear browsing data" -> clearBrowsingData()
                "Text size" -> textSizeDialog()
                "Dark pages: on", "Dark pages: off" -> toggleDarkPages()
                "Set as default" -> openDefaultSettings()
            }
            true
        }
        p.show()
    }
    private fun newPrivateTab() {
        snapThumb()
        tabs.add(Tab("", "Private", true, priv = true))
        tabIndex = tabs.lastIndex
        showHome()
        Toast.makeText(this, "Private tab — separate cookies, nothing saved", Toast.LENGTH_SHORT).show()
    }
    private fun openInNewTab(url: String) {
        snapThumb()
        tabs.add(Tab(url, url, false, priv = tabs[tabIndex].priv))
        tabIndex = tabs.lastIndex
        go(url)
    }
    private fun switchTab(i: Int) {
        if (i != tabIndex) snapThumb()
        tabIndex = i
        val t = tabs[i]
        renderOmniboxDecor(if (t.home) null else t.url)
        if (t.home || t.url.isEmpty()) showHome() else go(t.url)
    }
    private fun tabGroupKey(t: Tab): String =
        t.group?.takeIf { it.isNotBlank() } ?: hostOf(t.url).ifBlank { "Other" }
    private fun tabGrid() {
        snapThumb()
        val box = LinearLayout(this)
        box.orientation = LinearLayout.VERTICAL
        box.setBackgroundColor(themeColor(R.attr.amniBgPrimary))
        box.setPadding(dp(10f), dp(8f), dp(10f), dp(8f))
        val rowWrap = LinearLayout(this)
        rowWrap.orientation = LinearLayout.VERTICAL
        val grouped = tabs.mapIndexed { i, t -> i to t }.groupBy { (_, t) -> tabGroupKey(t) }
        fun paintGrid(dlg: android.app.AlertDialog) {
            rowWrap.removeAllViews()
            for ((groupName, entries) in grouped.entries.sortedBy { it.key.lowercase() }) {
                val hdr = TextView(this)
                hdr.text = groupName
                hdr.textSize = 12f
                hdr.setTextColor(themeColor(R.attr.amniTextSecondary))
                hdr.setPadding(dp(4f), dp(10f), dp(4f), dp(6f))
                rowWrap.addView(hdr)
                var row: LinearLayout? = null
                var col = 0
                entries.forEach { (i, t) ->
                    if (col % 2 == 0) {
                        row = LinearLayout(this).also {
                            it.orientation = LinearLayout.HORIZONTAL
                            it.setPadding(0, 0, 0, dp(8f))
                            rowWrap.addView(it)
                        }
                    }
                    val active = i == tabIndex
                    val cell = LinearLayout(this)
                    cell.orientation = LinearLayout.VERTICAL
                    cell.setBackgroundResource(if (active) R.drawable.tab_tile_active else R.drawable.tab_tile)
                    cell.setPadding(dp(8f), dp(8f), dp(8f), dp(8f))
                    val cLp = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
                    if (col % 2 == 0) cLp.marginEnd = dp(4f) else cLp.marginStart = dp(4f)
                    cell.layoutParams = cLp
                    val top = LinearLayout(this)
                    top.orientation = LinearLayout.HORIZONTAL
                    top.gravity = Gravity.CENTER_VERTICAL
                    val favIv = ImageView(this)
                    val fav = t.icon
                    if (fav != null && !t.home) favIv.setImageBitmap(fav)
                    else {
                        favIv.setBackgroundColor(themeColor(R.attr.amniBgHover))
                        favIv.setImageDrawable(null)
                    }
                    favIv.layoutParams = LinearLayout.LayoutParams(dp(18f), dp(18f)).also { it.marginEnd = dp(6f) }
                    top.addView(favIv)
                    val titleTv = TextView(this)
                    titleTv.text = (if (t.priv) "🕶 " else "") + (t.title.ifBlank { "New tab" }).take(32)
                    titleTv.setTextColor(if (active) themeColor(R.attr.amniAccent) else themeColor(R.attr.amniTextPrimary))
                    titleTv.textSize = 13f
                    titleTv.maxLines = 1
                    titleTv.ellipsize = android.text.TextUtils.TruncateAt.END
                    titleTv.layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
                    top.addView(titleTv)
                    val close = TextView(this)
                    close.text = "✕"
                    close.textSize = 14f
                    close.setTextColor(themeColor(R.attr.amniTextSecondary))
                    close.setPadding(dp(8f), dp(2f), dp(4f), dp(2f))
                    close.setOnClickListener { closeTabAt(i); dlg.dismiss() }
                    top.addView(close)
                    cell.addView(top)
                    val img = ImageView(this)
                    val bmp = t.thumb
                    if (bmp != null) img.setImageBitmap(bmp)
                    else img.setBackgroundColor(themeColor(R.attr.amniBgPrimary))
                    img.layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, dp(120f)).also {
                        it.topMargin = dp(6f)
                    }
                    img.scaleType = ImageView.ScaleType.CENTER_CROP
                    img.clipToOutline = true
                    img.outlineProvider = object : android.view.ViewOutlineProvider() {
                        override fun getOutline(view: View, outline: android.graphics.Outline) {
                            outline.setRoundRect(0, 0, view.width, view.height, dp(8f).toFloat())
                        }
                    }
                    cell.addView(img)
                    cell.setOnClickListener { switchTab(i); dlg.dismiss() }
                    cell.setOnLongClickListener { showTabGroupMenu(i, cell); true }
                    row!!.addView(cell)
                    col++
                }
                if (entries.size % 2 == 1) {
                    val spacer = View(this)
                    spacer.layoutParams = LinearLayout.LayoutParams(0, 0, 1f).also { it.marginStart = dp(4f) }
                    row!!.addView(spacer)
                }
            }
        }
        val sc = android.widget.ScrollView(this)
        sc.addView(rowWrap)
        box.addView(sc, LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, dp(480f)))
        val dlg = android.app.AlertDialog.Builder(this)
            .setTitle("Tabs (${tabs.size})")
            .setView(box)
            .setPositiveButton("New tab") { _, _ -> newTab() }
            .setNegativeButton("Close", null)
            .create()
        paintGrid(dlg)
        dlg.show()
        dlg.window?.setBackgroundDrawable(GradientDrawable().apply {
            setColor(themeColor(R.attr.amniBgPrimary))
            cornerRadius = dp(16f).toFloat()
        })
        dlg.findViewById<TextView>(android.R.id.title)?.setTextColor(themeColor(R.attr.amniTextPrimary))
        dlg.getButton(android.app.AlertDialog.BUTTON_POSITIVE)?.setTextColor(themeColor(R.attr.amniAccent))
        dlg.getButton(android.app.AlertDialog.BUTTON_NEGATIVE)?.setTextColor(themeColor(R.attr.amniTextSecondary))
    }
    private fun showTabGroupMenu(tabIdx: Int, anchor: View) {
        val pm = PopupMenu(this, anchor)
        pm.menu.add("New group…")
        tabs.mapNotNull { it.group?.takeIf { g -> g.isNotBlank() } }.distinct().forEach { g ->
            pm.menu.add("Group: $g")
        }
        if (tabs.getOrNull(tabIdx)?.group != null) pm.menu.add("Remove from group")
        pm.menu.add("Reorder")
        pm.setOnMenuItemClickListener { item ->
            when (val title = item.title.toString()) {
                "New group…" -> {
                    val input = EditText(this)
                    input.hint = "Group name"
                    android.app.AlertDialog.Builder(this).setTitle("New tab group").setView(input)
                        .setPositiveButton("Create") { _, _ ->
                            val name = input.text.toString().trim()
                            if (name.isNotEmpty()) assignGroup(tabIdx, name)
                        }
                        .setNegativeButton("Cancel", null).show()
                }
                "Remove from group" -> assignGroup(tabIdx, null)
                "Reorder" -> {
                    val clip = android.content.ClipData.newPlainText("tab", tabIdx.toString())
                    anchor.startDragAndDrop(clip, View.DragShadowBuilder(anchor), tabIdx, 0)
                }
                else -> if (title.startsWith("Group: ")) {
                    assignGroup(tabIdx, title.removePrefix("Group: "))
                }
            }
            true
        }
        pm.show()
    }
    private fun tabSizeDialog() {
        val opts = arrayOf("Compact", "Default", "Large")
        val ids = arrayOf("compact", "default", "large")
        val cur = ids.indexOf(ui.getString("tabsize", "default")).coerceAtLeast(0)
        android.app.AlertDialog.Builder(this).setTitle("Tab size")
            .setSingleChoiceItems(opts, cur) { d, i ->
                ui.edit().putString("tabsize", ids[i]).apply()
                applyTabStripHeight()
                paintTabs()
                updateChromeOutline()
                d.dismiss()
            }.show()
    }
    private fun themeDialog() {
        val opts = arrayOf(
            "Amni Scient — brass",
            "Amni Browse — emerald",
            "Amni — neon",
            "Amni Haven — violet",
            "Amni Crypt — blue",
            "Amni AI — amber",
            "Amni Core — red",
            "Amni Explore — cyan",
            "Amni Calc — orange",
            "Amni Learn — green",
            "Braid — lavender",
            "Amni Scient — light",
        )
        val ids = arrayOf("scient", "emerald", "amni", "haven", "crypt", "ai", "core", "explore", "calc", "learn", "braid", "light")
        val cur = ids.indexOf(ui.getString("theme", "scient")).coerceAtLeast(0)
        android.app.AlertDialog.Builder(this).setTitle("Browser theme")
            .setSingleChoiceItems(opts, cur) { d, i ->
                ui.edit().putString("theme", ids[i]).apply()
                d.dismiss()
                recreate()
            }.show()
    }
    private fun togglePullRefresh() {
        val on = !pullRefreshEnabled()
        ui.edit().putBoolean("pullrefresh", on).apply()
        applyPullRefresh()
        Toast.makeText(this, if (on) "Pull to refresh on" else "Pull to refresh off", Toast.LENGTH_SHORT).show()
    }
    private fun toggleJsBlock() {
        val host = hostOf(tabs[tabIndex].url)
        if (host.isEmpty()) return
        val set = jsBlocked()
        val nowBlocked = if (host in set) { set.remove(host); false } else { set.add(host); true }
        ui.edit().putStringSet("jsblock", set).apply()
        Toast.makeText(this, (if (nowBlocked) "JavaScript blocked on " else "JavaScript allowed on ") + host, Toast.LENGTH_SHORT).show()
        web.reload()
    }
    private fun toggleCookieBlock() {
        val host = hostOf(tabs[tabIndex].url)
        if (host.isEmpty()) return
        val set = cookieBlocked()
        val nowBlocked = if (host in set) { set.remove(host); false } else { set.add(host); true }
        ui.edit().putStringSet("cookblock", set).apply()
        if (nowBlocked) wipeCookies(tabs[tabIndex].url)
        Toast.makeText(this, (if (nowBlocked) "Cookies blocked on " else "Cookies allowed on ") + host, Toast.LENGTH_SHORT).show()
        web.reload()
    }
    private fun applyCookieGate(url: String) {
        val host = hostOf(url)
        val accept = CookieHosts.acceptCookies(host, cookieBlocked())
        val cm = CookieManager.getInstance()
        if (!accept) wipeCookies(url)
        cm.setAcceptCookie(accept)
    }
    private fun wipeCookies(url: String) {
        if (url.isEmpty()) return
        val cm = CookieManager.getInstance()
        for (n in CookieHosts.names(cm.getCookie(url))) cm.setCookie(url, CookieHosts.expirePair(n))
        cm.flush()
    }
    private fun fetchSuggest(q: String): List<String> {
        val u = SearchEngine.suggestUrl(engineId(), q) ?: return emptyList()
        return try {
            val conn = URL(u).openConnection() as HttpURLConnection
            conn.connectTimeout = 2000
            conn.readTimeout = 2000
            conn.setRequestProperty("User-Agent", "AmniBrowse")
            conn.inputStream.bufferedReader().use { SearchEngine.parseSuggest(engineId(), it.readText()) }.take(6)
        } catch (_: Exception) { emptyList() }
    }
    private fun snapThumb() {
        val t = tabs.getOrNull(tabIndex) ?: return
        if (homePanel.visibility == View.VISIBLE || web.width < 8 || web.height < 8) return
        val src = Bitmap.createBitmap(web.width.coerceAtMost(640), web.height.coerceAtMost(960), Bitmap.Config.RGB_565)
        val c = Canvas(src)
        c.scale(src.width / web.width.toFloat(), src.height / web.height.toFloat())
        web.draw(c)
        t.thumb = Bitmap.createScaledBitmap(src, dp(140f), dp(90f), true)
        if (t.thumb !== src) src.recycle()
    }
    private fun hideCustomView() {
        (customView?.parent as? ViewGroup)?.removeView(customView)
        customCb?.onCustomViewHidden()
        customView = null
        customCb = null
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
    }
    private fun enterPip() {
        if (Build.VERSION.SDK_INT < 26) { Toast.makeText(this, "PiP needs Android 8+", Toast.LENGTH_SHORT).show(); return }
        enterPictureInPictureMode(PictureInPictureParams.Builder().setAspectRatio(Rational(16, 9)).build())
    }
    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        if (customView != null && Build.VERSION.SDK_INT >= 26) enterPip()
    }
    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        val vis = if (isInPictureInPictureMode) View.GONE else View.VISIBLE
        findViewById<View>(R.id.chromeShell).visibility = vis
        loadPulse.visibility = if (isInPictureInPictureMode) View.GONE else loadPulse.visibility
    }
    private fun searchEngineDialog() {
        val names = SearchEngine.all.map { it.name }.toTypedArray()
        val cur = SearchEngine.all.indexOfFirst { it.id == engineId() }.coerceAtLeast(0)
        android.app.AlertDialog.Builder(this).setTitle("Search engine")
            .setSingleChoiceItems(names, cur) { d, i ->
                ui.edit().putString("engine", SearchEngine.all[i].id).apply()
                d.dismiss()
            }.show()
    }
    private fun textSizeDialog() {
        val opts = arrayOf("75%", "100%", "125%", "150%")
        val vals = intArrayOf(75, 100, 125, 150)
        val cur = vals.indexOf(ui.getInt("textzoom", 100)).coerceAtLeast(0)
        android.app.AlertDialog.Builder(this).setTitle("Text size")
            .setSingleChoiceItems(opts, cur) { d, i ->
                ui.edit().putInt("textzoom", vals[i]).apply()
                web.settings.textZoom = vals[i]
                d.dismiss()
            }.show()
    }
    private fun toggleDarkPages() {
        val on = !ui.getBoolean("darkpages", false)
        ui.edit().putBoolean("darkpages", on).apply()
        if (WebViewFeature.isFeatureSupported(WebViewFeature.ALGORITHMIC_DARKENING)) {
            WebSettingsCompat.setAlgorithmicDarkeningAllowed(web.settings, on)
            web.reload()
        } else Toast.makeText(this, "This WebView build cannot darken pages", Toast.LENGTH_LONG).show()
    }
    private fun printPage() {
        if (tabs[tabIndex].url.isEmpty()) return
        val pm = getSystemService(Context.PRINT_SERVICE) as PrintManager
        pm.print(tabs[tabIndex].title.ifBlank { "page" }, web.createPrintDocumentAdapter(tabs[tabIndex].title.ifBlank { "page" }), PrintAttributes.Builder().build())
    }
    private fun translatePage() {
        val u = tabs[tabIndex].url
        if (u.isEmpty()) return
        go("https://translate.google.com/translate?sl=auto&tl=en&u=" + URLEncoder.encode(u, "UTF-8"))
    }
    private fun readerMode() {
        if (tabs[tabIndex].url.isEmpty()) return
        web.evaluateJavascript(
            "(function(){var a=document.querySelector('article')||document.querySelector('main')||document.body;" +
            "return JSON.stringify({t:document.title,b:a?a.innerText:''})})()") { raw ->
            try {
                val o = JSONObject(JSONObject("{\"w\":" + raw + "}").getString("w"))
                val body = o.optString("b").take(200_000)
                if (body.isBlank()) { Toast.makeText(this, "Nothing readable found", Toast.LENGTH_SHORT).show(); return@evaluateJavascript }
                val paras = body.split("\n").filter { it.isNotBlank() }.joinToString("") { "<p>" + it.replace("&", "&amp;").replace("<", "&lt;") + "</p>" }
                val html = "<html><head><meta name=viewport content='width=device-width,initial-scale=1'><style>" +
                    "body{background:#08090B;color:#EDEFF2;font-family:serif;line-height:1.65;max-width:42em;margin:0 auto;padding:24px 20px}" +
                    "h1{color:#C89B4E;font-size:1.4em;line-height:1.3}p{margin:0 0 1em}</style></head><body><h1>" +
                    o.optString("t") + "</h1>" + paras + "</body></html>"
                web.loadDataWithBaseURL(tabs[tabIndex].url, html, "text/html", "utf-8", tabs[tabIndex].url)
            } catch (_: Exception) { Toast.makeText(this, "Reader failed on this page", Toast.LENGTH_SHORT).show() }
        }
    }
    private fun toggleBmBar() {
        val on = !ui.getBoolean("bmbar", false)
        ui.edit().putBoolean("bmbar", on).apply()
        bmBarWrap.visibility = if (on) View.VISIBLE else View.GONE
        if (on) paintBmBar()
    }
    private fun paintBmBar() {
        lifecycleScope.launch {
            val rows = withContext(Dispatchers.IO) { db.dao().allBookmarks() }
            bmBar.removeAllViews()
            for ((folder, items) in BookmarkFolders.group(rows)) {
                if (items.size == 1 && folder == "Bookmarks") {
                    val b = items[0]
                    val tv = TextView(this@BrowseActivity)
                    tv.text = b.title.ifBlank { b.url }.take(16)
                    tv.textSize = 11.5f
                    tv.setTextColor(themeColor(R.attr.amniTextSecondary))
                    tv.setPadding(22, 6, 22, 6)
                    tv.setOnClickListener { go(b.url) }
                    bmBar.addView(tv)
                } else {
                    val tv = TextView(this@BrowseActivity)
                    tv.text = "▾ $folder"
                    tv.textSize = 11.5f
                    tv.setTextColor(themeColor(R.attr.amniAccent))
                    tv.setPadding(22, 6, 22, 6)
                    tv.setOnClickListener { a ->
                        val pm = PopupMenu(this@BrowseActivity, a)
                        items.forEach { b -> pm.menu.add(b.title.ifBlank { b.url }) }
                        pm.setOnMenuItemClickListener { mi ->
                            items.find { it.title.ifBlank { it.url } == mi.title.toString() }?.let { go(it.url) }
                            true
                        }
                        pm.show()
                    }
                    bmBar.addView(tv)
                }
            }
        }
    }
    private fun bookmarksManager() {
        lifecycleScope.launch {
            val rows = withContext(Dispatchers.IO) { db.dao().allBookmarks() }
            if (rows.isEmpty()) { Toast.makeText(this@BrowseActivity, "No bookmarks yet", Toast.LENGTH_SHORT).show(); return@launch }
            val labels = rows.map { (if (it.path.isBlank()) "" else it.path + " › ") + it.title.ifBlank { it.url } }
            android.app.AlertDialog.Builder(this@BrowseActivity)
                .setTitle("Bookmarks (${rows.size}) — tap open, long-press edit")
                .setItems(labels.toTypedArray()) { _, i -> go(rows[i].url) }
                .setNegativeButton("Close", null)
                .create().also { dlg ->
                    dlg.setOnShowListener {
                        dlg.listView?.setOnItemLongClickListener { _, _, i, _ ->
                            dlg.dismiss(); editBookmark(rows[i]); true
                        }
                    }
                    dlg.show()
                }
        }
    }
    private fun clearBrowsingData() {
        val opts = arrayOf("History", "Cookies", "Cache")
        val picks = booleanArrayOf(true, false, true)
        android.app.AlertDialog.Builder(this).setTitle("Clear browsing data")
            .setMultiChoiceItems(opts, picks) { _, i, on -> picks[i] = on }
            .setPositiveButton("Clear") { _, _ ->
                lifecycleScope.launch {
                    if (picks[0]) withContext(Dispatchers.IO) { db.dao().clearHistory() }
                    if (picks[1]) CookieManager.getInstance().removeAllCookies(null)
                    if (picks[2]) { web.clearCache(true); web.clearFormData() }
                    Toast.makeText(this@BrowseActivity, "Cleared", Toast.LENGTH_SHORT).show()
                }
            }
            .setNegativeButton("Cancel", null).show()
    }
    override fun onCreateContextMenu(menu: ContextMenu, v: View, info: ContextMenu.ContextMenuInfo?) {
        super.onCreateContextMenu(menu, v, info)
        val hit = web.hitTestResult
        val url = hit.extra ?: return
        val isImg = hit.type == WebView.HitTestResult.IMAGE_TYPE || hit.type == WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE
        val isLink = hit.type == WebView.HitTestResult.SRC_ANCHOR_TYPE || hit.type == WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE
        if (!isImg && !isLink) return
        if (isLink) {
            menu.add("Open in new tab").setOnMenuItemClickListener { openInNewTab(url); true }
            menu.add("Copy link").setOnMenuItemClickListener {
                (getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager).setPrimaryClip(ClipData.newPlainText("url", url)); true }
            menu.add("Share link").setOnMenuItemClickListener {
                startActivity(Intent.createChooser(Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, url), "Share link")); true }
        }
        if (isImg) menu.add("Download image").setOnMenuItemClickListener {
            try {
                val req = DownloadManager.Request(Uri.parse(url))
                req.setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                req.setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, URLUtil.guessFileName(url, null, null))
                (getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager).enqueue(req)
            } catch (_: Exception) {}
            true
        }
    }
    private fun editBookmark(b: BookmarkEntity) {
        val box = LinearLayout(this)
        box.orientation = LinearLayout.VERTICAL
        box.setPadding(dp(20f), dp(8f), dp(20f), dp(4f))
        val title = EditText(this); title.setText(b.title); title.hint = "Title"
        val url = EditText(this); url.setText(b.url); url.hint = "URL"
        val path = EditText(this); path.setText(b.path); path.hint = "Folder"
        box.addView(title); box.addView(url); box.addView(path)
        android.app.AlertDialog.Builder(this).setTitle("Edit bookmark").setView(box)
            .setPositiveButton("Save") { _, _ ->
                lifecycleScope.launch(Dispatchers.IO) {
                    val nu = NavResolver.normalizeUrl(url.text.toString())
                    if (nu != b.url) db.dao().deleteBookmark(b.url)
                    if (nu.startsWith("http")) {
                        db.dao().deleteBookmark(nu)
                        db.dao().insertBookmark(BookmarkEntity(nu, title.text.toString().ifBlank { nu }, path.text.toString(), b.added))
                    }
                    withContext(Dispatchers.Main) { refreshBookmarks(); if (ui.getBoolean("bmbar", false)) paintBmBar() }
                }
            }
            .setNeutralButton("Delete") { _, _ ->
                lifecycleScope.launch(Dispatchers.IO) {
                    db.dao().deleteBookmark(b.url)
                    withContext(Dispatchers.Main) { refreshBookmarks(); if (ui.getBoolean("bmbar", false)) paintBmBar() }
                }
            }
            .setNegativeButton("Cancel", null).show()
    }
    private fun addBookmark() {
        val t = tabs[tabIndex]
        if (t.home || t.url.isEmpty()) { Toast.makeText(this, "Open a page first", Toast.LENGTH_SHORT).show(); return }
        lifecycleScope.launch(Dispatchers.IO) {
            db.dao().insertBookmark(BookmarkEntity(NavResolver.normalizeUrl(t.url), t.title.ifBlank { t.url }, "", System.currentTimeMillis()))
            withContext(Dispatchers.Main) { Toast.makeText(this@BrowseActivity, "Bookmarked", Toast.LENGTH_SHORT).show(); refreshBookmarks() }
        }
    }
    private fun sharePage() {
        val t = tabs[tabIndex]
        if (t.url.isEmpty()) return
        startActivity(Intent.createChooser(Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, t.url).putExtra(Intent.EXTRA_SUBJECT, t.title), "Share page"))
    }
    private fun toggleDesktop() {
        desktopSite = !desktopSite
        web.settings.userAgentString = if (desktopSite)
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36"
        else mobileUA
        web.settings.useWideViewPort = true
        web.settings.loadWithOverviewMode = desktopSite
        if (homePanel.visibility != View.VISIBLE) web.reload()
    }
    private fun findInPage() {
        val box = LinearLayout(this)
        box.orientation = LinearLayout.HORIZONTAL
        box.setPadding(24, 12, 24, 12)
        val input = EditText(this)
        input.hint = "Find in page"
        input.layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
        box.addView(input)
        val dlg = android.app.AlertDialog.Builder(this)
            .setView(box)
            .setPositiveButton("Next", null)
            .setNeutralButton("Previous", null)
            .setNegativeButton("Done") { _, _ -> web.clearMatches() }
            .setOnCancelListener { web.clearMatches() }
            .create()
        dlg.show()
        input.addTextChangedListener(object : TextWatcher {
            override fun afterTextChanged(s: Editable?) { web.findAllAsync(s?.toString() ?: "") }
            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) {}
            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) {}
        })
        web.setFindListener { active, total, done -> if (done) dlg.setTitle(if (total == 0) "No matches" else "${active + 1} / $total") }
        dlg.getButton(android.app.AlertDialog.BUTTON_POSITIVE)?.setOnClickListener { web.findNext(true) }
        dlg.getButton(android.app.AlertDialog.BUTTON_NEUTRAL)?.setOnClickListener { web.findNext(false) }
        input.requestFocus()
    }
    private fun scanAutoImport() {
        val prefs = getSharedPreferences("amni_autoimport", Context.MODE_PRIVATE)
        // Zero-permission automated lane: anything dropped into the app's own external
        // import/ dir (adb push, PC sync script) imports itself on next launch.
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val auto = getExternalFilesDir("import") ?: return@launch
                for (f in auto.listFiles() ?: emptyArray()) {
                    if (!f.isFile || f.length() > 20_000_000L) continue
                    val stampKey = "adone:" + f.name + ":" + f.lastModified()
                    if (prefs.getBoolean(stampKey, false)) continue
                    val text = f.readText()
                    withContext(Dispatchers.Main) { importText(text, f.name) }
                    prefs.edit().putBoolean(stampKey, true).apply()
                }
            } catch (_: Exception) {}
        }
        val tree = prefs.getString("tree", null) ?: return
        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val dir = DocumentFile.fromTreeUri(this@BrowseActivity, Uri.parse(tree)) ?: return@launch
                var found = 0
                for (f in dir.listFiles()) {
                    val name = f.name ?: continue
                    if (!f.isFile || f.length() > 20_000_000L) continue
                    if (!ImportParser.looksLikeBookmarkFile(name)) continue
                    val stampKey = "done:" + name + ":" + f.lastModified()
                    if (prefs.getBoolean(stampKey, false)) continue
                    val text = contentResolver.openInputStream(f.uri)?.bufferedReader()?.readText() ?: continue
                    withContext(Dispatchers.Main) { importText(text, name) }
                    prefs.edit().putBoolean(stampKey, true).apply()
                    found++
                }
                if (found > 0) withContext(Dispatchers.Main) { Toast.makeText(this@BrowseActivity, "Auto-imported $found export file(s)", Toast.LENGTH_LONG).show() }
            } catch (_: Exception) {}
        }
    }
    override fun onStart() {
        super.onStart()
        scanAutoImport()
    }
    private fun openDefaultSettings() {
        try {
            startActivity(Intent(Settings.ACTION_MANAGE_DEFAULT_APPS_SETTINGS))
        } catch (_: ActivityNotFoundException) {
            try {
                startActivity(Intent(Settings.ACTION_SETTINGS))
            } catch (_: ActivityNotFoundException) {
                Toast.makeText(this, "Open system settings → default browser", Toast.LENGTH_LONG).show()
            }
        }
    }
    private fun importUri(uri: Uri) {
        lifecycleScope.launch {
            try {
                val text = withContext(Dispatchers.IO) {
                    contentResolver.openInputStream(uri)?.bufferedReader()?.readText() ?: throw IllegalArgumentException("empty")
                }
                importText(text, uri.lastPathSegment ?: "file")
            } catch (e: Exception) {
                Toast.makeText(this@BrowseActivity, e.message ?: "import failed", Toast.LENGTH_LONG).show()
            }
        }
    }
    private fun importText(text: String, label: String) {
        lifecycleScope.launch {
            try {
                val parsed = ImportParser.parse(text)
                val dao = db.dao()
                var bm = 0
                var hist = 0
                withContext(Dispatchers.IO) {
                    for (chunk in parsed.bookmarks.chunked(500)) {
                        for (b in chunk) {
                            val n = dao.insertBookmark(BookmarkEntity(b.url, b.title, b.path, b.added))
                            if (n > 0) bm++
                        }
                    }
                    for (chunk in parsed.history.chunked(500)) {
                        for (h in chunk) {
                            val existing = dao.historyLast(h.url)
                            if (ImportParser.shouldUpdateHistory(existing, h.lastVisit)) {
                                if (existing == null) dao.insertHistory(HistoryEntity(h.url, h.title, h.lastVisit, h.visitCount))
                                else dao.touchHistory(h.url, h.title, h.lastVisit, 0)
                                hist++
                            }
                        }
                    }
                }
                android.util.Log.i("AmniImport", "imported bm=$bm hist=$hist from=$label source=${parsed.source}")
                Toast.makeText(this@BrowseActivity, "Imported $bm bookmarks and $hist history entries", Toast.LENGTH_LONG).show()
                refreshBookmarks()
            } catch (e: Exception) {
                Toast.makeText(this@BrowseActivity, e.message ?: "import failed", Toast.LENGTH_LONG).show()
            }
        }
    }
    private fun recordHistory(url: String?, title: String?) {
        if (url.isNullOrBlank() || !url.startsWith("http")) return
        val key = NavResolver.normalizeUrl(url)
        val t = title ?: key
        val now = System.currentTimeMillis()
        lifecycleScope.launch(Dispatchers.IO) {
            val dao = db.dao()
            val n = dao.touchHistory(key, t, now, 1)
            if (n == 0) dao.insertHistory(HistoryEntity(key, t, now, 1))
        }
    }
    private fun twoLineAdapter(rows: List<Pair<String, String>>): ArrayAdapter<Pair<String, String>> =
        object : ArrayAdapter<Pair<String, String>>(this, 0, rows) {
            override fun getView(pos: Int, convert: View?, parent: android.view.ViewGroup): View {
                val (title, sub) = rows[pos]
                val box = (convert as? LinearLayout) ?: LinearLayout(context).also {
                    it.orientation = LinearLayout.VERTICAL
                    it.setPadding(dp(4f), dp(9f), dp(4f), dp(9f))
                    it.addView(TextView(context).apply { textSize = 14.5f; setTextColor(themeColor(R.attr.amniTextPrimary)); maxLines = 1; ellipsize = android.text.TextUtils.TruncateAt.END })
                    it.addView(TextView(context).apply { textSize = 11.5f; setTextColor(themeColor(R.attr.amniTextSecondary)); maxLines = 1; ellipsize = android.text.TextUtils.TruncateAt.END })
                }
                (box.getChildAt(0) as TextView).text = title
                (box.getChildAt(1) as TextView).text = sub
                return box
            }
        }
    private fun refreshBookmarks() {
        lifecycleScope.launch {
            val rows = withContext(Dispatchers.IO) { db.dao().allBookmarks() }
            findViewById<View>(R.id.bookmarkEmpty).visibility = if (rows.isEmpty()) View.VISIBLE else View.GONE
            bookmarkList.adapter = twoLineAdapter(rows.map { (it.title.ifBlank { it.url }) to hostOf(it.url) })
            bookmarkList.setOnItemClickListener { _, _, pos, _ -> go(rows[pos].url) }
        }
        if (ui.getBoolean("bmbar", false)) paintBmBar()
    }
    private fun showHistory() {
        lifecycleScope.launch {
            var rows = withContext(Dispatchers.IO) { db.dao().recentHistory() }
            val box = LinearLayout(this@BrowseActivity)
            box.orientation = LinearLayout.VERTICAL
            box.setPadding(24, 8, 24, 0)
            val search = EditText(this@BrowseActivity)
            search.hint = "Search history"
            box.addView(search)
            val list = ListView(this@BrowseActivity)
            box.addView(list, LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, dp(360f)))
            fun paint() { list.adapter = ArrayAdapter(this@BrowseActivity, android.R.layout.simple_list_item_1, rows.map { it.title.ifBlank { it.url } + "\n" + it.url }) }
            paint()
            val dlg = android.app.AlertDialog.Builder(this@BrowseActivity)
                .setTitle("History")
                .setView(box)
                .setNegativeButton("Close", null)
                .setNeutralButton("Clear all") { _, _ ->
                    android.app.AlertDialog.Builder(this@BrowseActivity).setTitle("Delete all history?")
                        .setPositiveButton("Delete") { _, _ -> lifecycleScope.launch(Dispatchers.IO) { db.dao().clearHistory() } }
                        .setNegativeButton("Keep", null).show()
                }
                .create()
            list.setOnItemClickListener { _, _, i, _ -> dlg.dismiss(); go(rows[i].url) }
            search.addTextChangedListener(object : TextWatcher {
                override fun afterTextChanged(s: Editable?) {
                    val q = s?.toString()?.trim() ?: ""
                    lifecycleScope.launch {
                        rows = withContext(Dispatchers.IO) { if (q.isEmpty()) db.dao().recentHistory() else db.dao().searchHistory(q, 200) }
                        paint()
                    }
                }
                override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) {}
                override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) {}
            })
            dlg.show()
        }
    }

    private companion object {
        const val STATE_TABS = "state_tabs"
        const val STATE_INDEX = "state_index"
    }
}
