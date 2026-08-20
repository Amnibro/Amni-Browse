package com.amniscient.browse
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import android.view.Gravity
import android.view.View
import android.view.autofill.AutofillManager
import android.view.inputmethod.EditorInfo
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
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
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
class BrowseActivity : AppCompatActivity() {
    private lateinit var web: WebView
    private lateinit var urlBar: EditText
    private lateinit var progress: ProgressBar
    private lateinit var homePanel: View
    private lateinit var missingWeb: View
    private lateinit var retryBar: View
    private lateinit var tabStrip: LinearLayout
    private lateinit var bookmarkList: ListView
    private lateinit var db: AppDb
    private val tabs = mutableListOf(Tab("", "Home", true))
    private var tabIndex = 0
    private var lastFailedUrl: String? = null
    private val openDoc = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) importUri(uri)
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_browse)
        db = AppDb.get(this)
        web = findViewById(R.id.web)
        urlBar = findViewById(R.id.urlBar)
        progress = findViewById(R.id.progress)
        homePanel = findViewById(R.id.homePanel)
        missingWeb = findViewById(R.id.missingWeb)
        retryBar = findViewById(R.id.retryBar)
        tabStrip = findViewById(R.id.tabStrip)
        bookmarkList = findViewById(R.id.bookmarkList)
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
        web.settings.javaScriptEnabled = true
        web.settings.domStorageEnabled = true
        web.importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_YES
        val af = getSystemService(AutofillManager::class.java)
        af?.notifyViewEntered(web)
        web.webViewClient = object : WebViewClient() {
            override fun onPageFinished(view: WebView?, url: String?) {
                retryBar.visibility = View.GONE
                if (url.isNullOrBlank() || url.startsWith("about:")) return
                tabs[tabIndex].url = url
                tabs[tabIndex].title = view?.title ?: tabs[tabIndex].title
                tabs[tabIndex].home = false
                if (!urlBar.hasFocus()) urlBar.setText(displayUrl(url))
                paintTabs()
                saveSession()
                recordHistory(url, view?.title)
            }
            override fun onReceivedError(view: WebView?, request: WebResourceRequest?, error: WebResourceError?) {
                if (request?.isForMainFrame == true) {
                    lastFailedUrl = request.url?.toString() ?: tabs[tabIndex].url
                    retryBar.visibility = View.VISIBLE
                }
            }
            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean = false
        }
        web.webChromeClient = object : WebChromeClient() {
            override fun onProgressChanged(view: WebView?, newProgress: Int) {
                progress.progress = newProgress
                progress.visibility = if (newProgress in 1..99) View.VISIBLE else View.GONE
            }
            override fun onReceivedTitle(view: WebView?, title: String?) {
                tabs[tabIndex].title = title ?: tabs[tabIndex].title
                paintTabs()
            }
        }
        findViewById<ImageButton>(R.id.btnBack).setOnClickListener { if (web.canGoBack()) web.goBack() }
        findViewById<ImageButton>(R.id.btnForward).setOnClickListener { if (web.canGoForward()) web.goForward() }
        findViewById<ImageButton>(R.id.btnHome).setOnClickListener { showHome() }
        findViewById<ImageButton>(R.id.btnMenu).setOnClickListener { showMenu(it) }
        findViewById<Button>(R.id.btnImport).setOnClickListener { openDoc.launch(arrayOf("application/json", "text/plain", "*/*")) }
        findViewById<Button>(R.id.btnDefault).setOnClickListener { openDefaultSettings() }
        findViewById<Button>(R.id.btnRetry).setOnClickListener { lastFailedUrl?.let { go(it) } }
        urlBar.setOnEditorActionListener { _, action, _ ->
            if (action == EditorInfo.IME_ACTION_GO || action == EditorInfo.IME_ACTION_DONE) {
                go(urlBar.text.toString()); true
            } else false
        }
        bookmarkList.setOnItemClickListener { _, _, pos, _ ->
            val item = bookmarkList.adapter.getItem(pos) as String
            go(item.substringAfter('\n'))
        }
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                if (web.visibility == View.VISIBLE && web.canGoBack()) web.goBack() else finish()
            }
        })
        SessionStore.load(this)?.let { (saved, idx) ->
            tabs.clear()
            tabs.addAll(saved)
            tabIndex = idx
        }
        handleIntent(intent)
        paintTabs()
        refreshBookmarks()
        val t = tabs[tabIndex]
        if (t.home || t.url.isEmpty()) showHome() else go(t.url)
    }
    override fun onStop() {
        super.onStop()
        saveSession()
    }
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }
    private fun saveSession() = SessionStore.save(this, tabs, tabIndex)
    private fun handleIntent(intent: Intent?) {
        val data = intent?.dataString
        if (intent?.action == Intent.ACTION_VIEW && !data.isNullOrBlank()) go(data)
    }
    private fun displayUrl(url: String?): String {
        val u = url ?: return ""
        if (u.startsWith("about:") || u.startsWith("data:")) return ""
        return u
    }
    private fun go(raw: String) {
        val url = NavResolver.resolve(raw)
        retryBar.visibility = View.GONE
        lastFailedUrl = url
        tabs[tabIndex].home = false
        tabs[tabIndex].url = url
        homePanel.visibility = View.GONE
        web.visibility = View.VISIBLE
        web.loadUrl(url)
        urlBar.setText(url)
        paintTabs()
        saveSession()
    }
    private fun showHome() {
        tabs[tabIndex].home = true
        tabs[tabIndex].title = "Home"
        tabs[tabIndex].url = ""
        web.stopLoading()
        homePanel.visibility = View.VISIBLE
        retryBar.visibility = View.GONE
        urlBar.setText("")
        refreshBookmarks()
        paintTabs()
        saveSession()
    }
    private fun newTab() {
        tabs.add(Tab("", "Home", true))
        tabIndex = tabs.lastIndex
        showHome()
    }
    private fun closeTab() {
        if (tabs.size == 1) { showHome(); return }
        tabs.removeAt(tabIndex)
        tabIndex = tabIndex.coerceAtMost(tabs.lastIndex)
        val t = tabs[tabIndex]
        if (t.home || t.url.isEmpty()) showHome() else go(t.url)
    }
    private fun paintTabs() {
        tabStrip.removeAllViews()
        tabs.forEachIndexed { i, t ->
            val tv = TextView(this)
            tv.text = (if (t.title.isBlank()) "Tab" else t.title).take(18)
            tv.setTextColor(if (i == tabIndex) getColor(R.color.accent) else getColor(R.color.text_secondary))
            tv.setPadding(24, 12, 24, 12)
            tv.setOnClickListener {
                tabIndex = i
                if (t.home || t.url.isEmpty()) showHome() else go(t.url)
            }
            tabStrip.addView(tv)
        }
        val plus = TextView(this)
        plus.text = "+"
        plus.setTextColor(getColor(R.color.accent))
        plus.setPadding(24, 12, 24, 12)
        plus.setOnClickListener { newTab() }
        plus.gravity = Gravity.CENTER
        tabStrip.addView(plus)
    }
    private fun showMenu(anchor: View) {
        val p = PopupMenu(this, anchor)
        p.menu.add("New tab")
        p.menu.add("Close tab")
        p.menu.add("Import Chrome file")
        p.menu.add("Set as default")
        p.menu.add("History")
        p.setOnMenuItemClickListener {
            when (it.title) {
                "New tab" -> newTab()
                "Close tab" -> closeTab()
                "Import Chrome file" -> openDoc.launch(arrayOf("application/json", "*/*"))
                "Set as default" -> openDefaultSettings()
                "History" -> showHistory()
            }
            true
        }
        p.show()
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
                Toast.makeText(this@BrowseActivity, "Imported $bm bookmarks, $hist history", Toast.LENGTH_LONG).show()
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
    private fun refreshBookmarks() {
        lifecycleScope.launch {
            val rows = withContext(Dispatchers.IO) { db.dao().allBookmarks() }
            val labels = rows.map { "${it.title}\n${it.url}" }
            bookmarkList.adapter = ArrayAdapter(this@BrowseActivity, android.R.layout.simple_list_item_1, labels)
        }
    }
    private fun showHistory() {
        lifecycleScope.launch {
            val rows = withContext(Dispatchers.IO) { db.dao().recentHistory() }
            val labels = rows.map { it.title.ifBlank { it.url } }
            android.app.AlertDialog.Builder(this@BrowseActivity)
                .setTitle("History")
                .setItems(labels.toTypedArray()) { _, i -> go(rows[i].url) }
                .setNegativeButton("Close", null)
                .show()
        }
    }
}
