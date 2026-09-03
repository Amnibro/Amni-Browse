package com.amniscient.browse
import org.json.JSONArray
import org.json.JSONObject
data class ImportedBookmark(val title: String, val url: String, val path: String, val added: Long)
data class ImportedHistory(val title: String, val url: String, val lastVisit: Long, val visitCount: Int)
data class ImportFile(val bookmarks: List<ImportedBookmark>, val history: List<ImportedHistory>, val source: String = "amni")
object ImportParser {
    private val banned = listOf("password", "cookie", "token")
    // Sniffs the real-world export formats: our own v1 JSON, Chrome/Edge/Brave desktop
    // "Bookmarks" (roots tree, WebKit-epoch microseconds), Firefox JSON backup
    // (moz-place tree, microseconds), and Netscape bookmarks HTML (what every browser's
    // Export button writes, Safari included).
    fun parse(text: String): ImportFile {
        val t = text.trimStart()
        if (t.startsWith("{")) {
            val low = text.lowercase()
            for (b in banned) if (low.contains("\"$b\"")) throw IllegalArgumentException("refusing import with banned key $b")
            val root = JSONObject(t)
            return when {
                root.has("roots") -> parseChromeRoots(root)
                root.optString("type").contains("moz-place") || (root.has("children") && root.has("guid")) -> parseFirefox(root)
                root.optInt("version", -1) == 1 -> parseV1(root)
                else -> throw IllegalArgumentException("unrecognized JSON export")
            }
        }
        if (t.contains("NETSCAPE-Bookmark-file", ignoreCase = true) || t.startsWith("<")) return parseNetscape(text)
        throw IllegalArgumentException("unrecognized format")
    }
    private fun parseV1(root: JSONObject): ImportFile {
        val bms = mutableListOf<ImportedBookmark>()
        val arr = root.optJSONArray("bookmarks") ?: JSONArray()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            val url = NavResolver.normalizeUrl(o.optString("url"))
            if (!url.startsWith("http")) continue
            val pathArr = o.optJSONArray("path")
            val path = if (pathArr == null) "" else (0 until pathArr.length()).joinToString("/") { pathArr.optString(it) }
            bms.add(ImportedBookmark(o.optString("title", url), url, path, o.optLong("added", 0)))
        }
        val hist = mutableListOf<ImportedHistory>()
        val harr = root.optJSONArray("history") ?: JSONArray()
        for (i in 0 until harr.length()) {
            val o = harr.getJSONObject(i)
            val url = NavResolver.normalizeUrl(o.optString("url"))
            if (!url.startsWith("http")) continue
            hist.add(ImportedHistory(o.optString("title", url), url, o.optLong("lastVisit", 0), o.optInt("visitCount", 0)))
        }
        return ImportFile(dedupeBookmarks(bms), dedupeHistory(hist), "amni")
    }
    private fun webkitToMs(us: Long): Long = if (us <= 0) 0 else us / 1000 - 11644473600000L
    private fun parseChromeRoots(root: JSONObject): ImportFile {
        val bms = mutableListOf<ImportedBookmark>()
        val roots = root.getJSONObject("roots")
        for (key in roots.keys()) {
            val node = roots.optJSONObject(key) ?: continue
            walkChrome(node, node.optString("name", key), bms)
        }
        return ImportFile(dedupeBookmarks(bms), emptyList(), "chrome")
    }
    private fun walkChrome(node: JSONObject, path: String, out: MutableList<ImportedBookmark>) {
        if (node.optString("type") == "url") {
            val url = NavResolver.normalizeUrl(node.optString("url"))
            if (url.startsWith("http")) out.add(ImportedBookmark(node.optString("name", url), url, path, webkitToMs(node.optString("date_added", "0").toLongOrNull() ?: 0)))
            return
        }
        val kids = node.optJSONArray("children") ?: return
        for (i in 0 until kids.length()) {
            val k = kids.optJSONObject(i) ?: continue
            val sub = if (k.optString("type") == "folder") (if (path.isEmpty()) k.optString("name") else path + "/" + k.optString("name")) else path
            walkChrome(k, sub, out)
        }
    }
    private fun parseFirefox(root: JSONObject): ImportFile {
        val bms = mutableListOf<ImportedBookmark>()
        walkMoz(root, "", bms)
        return ImportFile(dedupeBookmarks(bms), emptyList(), "firefox")
    }
    private fun walkMoz(node: JSONObject, path: String, out: MutableList<ImportedBookmark>) {
        val uri = node.optString("uri")
        if (uri.isNotEmpty()) {
            val url = NavResolver.normalizeUrl(uri)
            if (url.startsWith("http")) out.add(ImportedBookmark(node.optString("title", url).ifEmpty { url }, url, path, node.optLong("dateAdded", 0) / 1000))
            return
        }
        val kids = node.optJSONArray("children") ?: return
        val name = node.optString("title")
        val here = when { path.isEmpty() -> name; name.isEmpty() -> path; else -> "$path/$name" }
        for (i in 0 until kids.length()) walkMoz(kids.optJSONObject(i) ?: continue, here, out)
    }
    private val NS_LINK = Regex("<DT><A[^>]*HREF=\"([^\"]+)\"[^>]*>(.*?)</A>", setOf(RegexOption.IGNORE_CASE, RegexOption.DOT_MATCHES_ALL))
    private val NS_DATE = Regex("ADD_DATE=\"(\\d+)\"", RegexOption.IGNORE_CASE)
    private val NS_FOLDER = Regex("<DT><H3[^>]*>(.*?)</H3>", setOf(RegexOption.IGNORE_CASE, RegexOption.DOT_MATCHES_ALL))
    private fun parseNetscape(text: String): ImportFile {
        val bms = mutableListOf<ImportedBookmark>()
        val stack = mutableListOf<String>()
        // Walk the tokens in document order so nested folder paths stay honest.
        val tokens = Regex("(<DT><H3[^>]*>.*?</H3>|<DT><A[^>]*>.*?</A>|</DL>)", setOf(RegexOption.IGNORE_CASE, RegexOption.DOT_MATCHES_ALL)).findAll(text)
        for (m in tokens) {
            val tok = m.value
            if (tok.equals("</DL>", ignoreCase = true)) { if (stack.isNotEmpty()) stack.removeAt(stack.lastIndex); continue }
            val f = NS_FOLDER.find(tok)
            if (f != null) { stack.add(unescape(f.groupValues[1]).trim()); continue }
            val a = NS_LINK.find(tok) ?: continue
            val url = NavResolver.normalizeUrl(a.groupValues[1])
            if (!url.startsWith("http")) continue
            val added = (NS_DATE.find(tok)?.groupValues?.get(1)?.toLongOrNull() ?: 0L).let { if (it > 0) it * 1000 else 0 }
            bms.add(ImportedBookmark(unescape(a.groupValues[2]).trim().ifEmpty { url }, url, stack.joinToString("/"), added))
        }
        return ImportFile(dedupeBookmarks(bms), emptyList(), "html")
    }
    private fun unescape(s: String): String = s
        .replace(Regex("<[^>]+>"), "")
        .replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
        .replace("&quot;", "\"").replace("&#39;", "'")
    fun looksLikeBookmarkFile(name: String): Boolean {
        val n = name.lowercase()
        return n == "bookmarks" || n.startsWith("bookmarks_") || (n.endsWith(".html") && n.contains("bookmark")) || (n.endsWith(".json") && n.contains("bookmark"))
    }
    fun shouldInsertBookmark(already: Boolean): Boolean = !already
    fun shouldUpdateHistory(existingLast: Long?, incomingLast: Long): Boolean {
        if (existingLast == null) return true
        return incomingLast > existingLast
    }
    private fun dedupeBookmarks(rows: List<ImportedBookmark>): List<ImportedBookmark> {
        val map = linkedMapOf<String, ImportedBookmark>()
        for (r in rows) if (!map.containsKey(r.url)) map[r.url] = r
        return map.values.toList()
    }
    private fun dedupeHistory(rows: List<ImportedHistory>): List<ImportedHistory> {
        val map = linkedMapOf<String, ImportedHistory>()
        for (r in rows) {
            val old = map[r.url]
            map[r.url] = if (old == null || r.lastVisit >= old.lastVisit) r else old
        }
        return map.values.toList()
    }
}
