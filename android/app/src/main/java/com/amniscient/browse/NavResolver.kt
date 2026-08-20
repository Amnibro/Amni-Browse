package com.amniscient.browse
object NavResolver {
    fun resolve(raw: String): String {
        val q = raw.trim()
        if (q.isEmpty()) return "https://duckduckgo.com/"
        val lower = q.lowercase()
        if (lower.startsWith("http://") || lower.startsWith("https://")) return q
        if (looksLikeHost(q)) return "https://$q"
        return "https://duckduckgo.com/?q=" + java.net.URLEncoder.encode(q, "UTF-8")
    }
    fun looksLikeHost(q: String): Boolean {
        if (q.contains(' ') || q.contains('\t')) return false
        val host = q.substringBefore('/').substringBefore(':')
        if (!host.contains('.')) return false
        if (host.startsWith('.') || host.endsWith('.')) return false
        return host.all { it.isLetterOrDigit() || it == '.' || it == '-' }
    }
    fun normalizeUrl(url: String): String {
        var u = url.trim()
        while (u.endsWith('/')) u = u.dropLast(1)
        return u.lowercase()
    }
}
