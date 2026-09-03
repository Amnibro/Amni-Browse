package com.amniscient.browse

import java.net.URI

enum class ConnectionSecurity {
    SECURE,
    INSECURE_HTTP,
    LOCAL_OR_INTERNAL,
    UNKNOWN,
}

enum class BrowserControl {
    BACK,
    FORWARD,
    RELOAD,
}

enum class BrowserControlResult {
    GO_BACK,
    GO_FORWARD,
    RELOAD,
    NONE,
}

/**
 * Pure browser-surface decisions kept outside Activity/WebView code so the
 * security and navigation behavior can be covered by local unit tests.
 */
object BrowserPolicies {
    private val externalSchemes = setOf(
        "mailto", "tel", "sms", "smsto", "geo", "market",
    )
    private val dangerousSchemes = setOf(
        "javascript", "vbscript", "file", "content",
    )

    /** Pause/kill HTML media before tearing down or leaving a page. Matches desktop trash path. */
    const val SILENCE_JS: String =
        "(function(){try{window.stop()}catch(e){}" +
            "try{document.querySelectorAll('video,audio').forEach(function(m){try{m.pause();m.removeAttribute('src');m.srcObject=null;m.load()}catch(e){}})}catch(e){}" +
            "try{if(window.speechSynthesis)speechSynthesis.cancel()}catch(e){}})()"

    fun connectionSecurity(url: String?): ConnectionSecurity {
        val scheme = schemeOf(url)
        return when (scheme) {
            "https" -> ConnectionSecurity.SECURE
            "http" -> ConnectionSecurity.INSECURE_HTTP
            "about", "data", "file", "content" -> ConnectionSecurity.LOCAL_OR_INTERNAL
            null -> ConnectionSecurity.UNKNOWN
            else -> ConnectionSecurity.UNKNOWN
        }
    }

    /** Schemes that must never load in the in-app WebView (XSS / local file exfil). */
    fun isDangerousScheme(url: String?): Boolean {
        val scheme = schemeOf(url) ?: return false
        return scheme in dangerousSchemes
    }

    fun canLoadInWebView(url: String?): Boolean {
        val scheme = schemeOf(url) ?: return false
        if (scheme in dangerousSchemes) return false
        return scheme == "http" || scheme == "https" || scheme == "about" || scheme == "data" || scheme == "blob"
    }

    fun canOpenExternally(url: String?): Boolean {
        val scheme = schemeOf(url)
        return scheme != null && (scheme == "intent" || scheme in externalSchemes)
    }

    fun canDownload(url: String?): Boolean {
        val scheme = schemeOf(url)
        return scheme != null && scheme in setOf("http", "https")
    }

    fun fileChooserAcceptTypes(raw: Array<out String>?): Array<String> =
        raw.orEmpty()
            .map(String::trim)
            .filter { it.isNotEmpty() }
            .distinct()
            .ifEmpty { listOf("*/*") }
            .toTypedArray()

    fun controlResult(
        control: BrowserControl,
        homeVisible: Boolean,
        canGoBack: Boolean,
        canGoForward: Boolean,
    ): BrowserControlResult = when (control) {
        BrowserControl.BACK -> if (!homeVisible && canGoBack) BrowserControlResult.GO_BACK else BrowserControlResult.NONE
        BrowserControl.FORWARD -> if (!homeVisible && canGoForward) BrowserControlResult.GO_FORWARD else BrowserControlResult.NONE
        BrowserControl.RELOAD -> if (!homeVisible) BrowserControlResult.RELOAD else BrowserControlResult.NONE
    }

    private fun schemeOf(url: String?): String? {
        if (url.isNullOrBlank()) return null
        return try {
            URI(url.trim()).scheme?.lowercase()
        } catch (_: Exception) {
            null
        }
    }
}

/**
 * Tracks a main-frame load independently from WebView's error document.
 * WebView calls onPageFinished for its generated error page, so finishing is
 * only success when the active attempt has not received a main-frame error.
 */
class MainFrameNavigationState {
    var failedUrl: String? = null
        private set

    private var activeUrl: String? = null
    private var activeAttemptFailed = false

    fun onPageStarted(url: String?) {
        if (url.isNullOrBlank() || url.startsWith("chrome-error://")) return
        activeUrl = url
        activeAttemptFailed = false
    }

    fun onMainFrameError(url: String?) {
        val failed = url?.takeIf { it.isNotBlank() } ?: activeUrl
        failedUrl = failed
        activeAttemptFailed = true
    }

    /**
     * Returns true only when this callback verifies a successful active load.
     */
    fun onPageFinished(url: String?): Boolean {
        if (url.isNullOrBlank() || activeUrl.isNullOrBlank()) return false
        if (url != activeUrl || activeAttemptFailed) return false
        failedUrl = null
        return true
    }

    fun hasFailure(): Boolean = failedUrl != null
}
