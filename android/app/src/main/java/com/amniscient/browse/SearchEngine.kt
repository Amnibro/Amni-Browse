package com.amniscient.browse
import org.json.JSONArray
import java.net.URLEncoder
object SearchEngine {
    data class Engine(val id: String, val name: String, val search: String, val suggest: String)
    val all = listOf(
        Engine("ddg", "DuckDuckGo", "https://duckduckgo.com/?q=%s", "https://duckduckgo.com/ac/?q=%s"),
        Engine("google", "Google", "https://www.google.com/search?q=%s", "https://suggestqueries.google.com/complete/search?client=firefox&q=%s"),
        Engine("bing", "Bing", "https://www.bing.com/search?q=%s", "https://api.bing.com/osjson.aspx?query=%s"),
        Engine("startpage", "Startpage", "https://www.startpage.com/sp/search?query=%s", ""),
        Engine("wikipedia", "Wikipedia", "https://en.wikipedia.org/w/index.php?search=%s", "https://en.wikipedia.org/w/api.php?action=opensearch&search=%s")
    )
    fun byId(id: String): Engine = all.find { it.id == id } ?: all[0]
    fun enc(q: String): String = URLEncoder.encode(q, "UTF-8")
    fun searchUrl(id: String, q: String): String = byId(id).search.replace("%s", enc(q))
    fun suggestUrl(id: String, q: String): String? {
        val s = byId(id).suggest
        return if (s.isEmpty()) null else s.replace("%s", enc(q))
    }
    fun parseSuggest(id: String, body: String): List<String> {
        if (body.isBlank()) return emptyList()
        return try {
            val arr = JSONArray(body)
            when (id) {
                "ddg" -> (0 until arr.length()).mapNotNull { arr.optJSONObject(it)?.optString("phrase")?.takeIf { p -> p.isNotBlank() } }
                else -> {
                    val inner = arr.optJSONArray(1) ?: return emptyList()
                    (0 until inner.length()).mapNotNull { inner.optString(it).takeIf { p -> p.isNotBlank() } }
                }
            }
        } catch (_: Exception) { emptyList() }
    }
}
