package com.amniscient.browse
object BookmarkFolders {
    fun firstFolder(path: String): String {
        val p = path.substringBefore('/').trim()
        return if (p.isEmpty()) "Bookmarks" else p
    }
    fun group(rows: List<BookmarkEntity>): List<Pair<String, List<BookmarkEntity>>> {
        val map = LinkedHashMap<String, MutableList<BookmarkEntity>>()
        for (b in rows) map.getOrPut(firstFolder(b.path)) { mutableListOf() }.add(b)
        return map.map { it.key to it.value.toList() }
    }
}
