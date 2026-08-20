package com.amniscient.browse
import org.json.JSONArray
import org.json.JSONObject
data class ImportedBookmark(val title: String, val url: String, val path: String, val added: Long)
data class ImportedHistory(val title: String, val url: String, val lastVisit: Long, val visitCount: Int)
data class ImportFile(val bookmarks: List<ImportedBookmark>, val history: List<ImportedHistory>)
object ImportParser {
    private val banned = listOf("password", "cookie", "token")
    fun parse(text: String): ImportFile {
        val low = text.lowercase()
        for (b in banned) {
            if (low.contains("\"$b\"")) throw IllegalArgumentException("refusing import with banned key $b")
        }
        val root = JSONObject(text)
        if (root.optInt("version", -1) != 1) throw IllegalArgumentException("unsupported version")
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
        return ImportFile(dedupeBookmarks(bms), dedupeHistory(hist))
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
