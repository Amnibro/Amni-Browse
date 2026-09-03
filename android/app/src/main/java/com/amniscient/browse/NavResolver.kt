package com.amniscient.browse
object NavResolver {
    private val trackers = setOf(
        "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content", "utm_id", "utm_cid",
        "fbclid", "gclid", "gclsrc", "dclid", "gbraid", "wbraid", "msclkid", "mc_eid", "mc_cid",
        "igshid", "twclid", "yclid", "_hsenc", "_hsmi", "mkt_tok", "wickedid", "oly_anon_id", "oly_enc_id"
    )
    fun resolve(raw: String, engineId: String = "ddg"): String {
        val q = raw.trim()
        if (q.isEmpty()) return "https://duckduckgo.com/"
        val lower = q.lowercase()
        if (lower.startsWith("http://") || lower.startsWith("https://")) return stripTrackers(q)
        if (looksLikeHost(q)) return stripTrackers("https://$q")
        return SearchEngine.searchUrl(engineId, q)
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
    fun stripTrackers(url: String): String {
        val qi = url.indexOf('?')
        if (qi < 0) return url
        val hash = url.indexOf('#', qi)
        val base = url.substring(0, qi)
        val frag = if (hash >= 0) url.substring(hash) else ""
        val query = url.substring(qi + 1, if (hash >= 0) hash else url.length)
        val kept = query.split('&').filter { part ->
            val name = part.substringBefore('=').lowercase()
            name.isNotEmpty() && name !in trackers
        }
        return if (kept.isEmpty()) base + frag else base + "?" + kept.joinToString("&") + frag
    }
}
