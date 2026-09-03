package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Test
class BookmarkFoldersTest {
    @Test fun groupsByFirstPathSegment() {
        val rows = listOf(
            BookmarkEntity("https://a", "A", "Work/Dev", 1),
            BookmarkEntity("https://b", "B", "Work/Ops", 2),
            BookmarkEntity("https://c", "C", "", 3),
            BookmarkEntity("https://d", "D", "News", 4)
        )
        val g = BookmarkFolders.group(rows)
        assertEquals(listOf("Work", "Bookmarks", "News"), g.map { it.first })
        assertEquals(2, g[0].second.size)
        assertEquals("C", g[1].second[0].title)
    }
    @Test fun firstFolderBlankIsBookmarks() {
        assertEquals("Bookmarks", BookmarkFolders.firstFolder(""))
        assertEquals("Bar", BookmarkFolders.firstFolder("Bar/x"))
    }
}
