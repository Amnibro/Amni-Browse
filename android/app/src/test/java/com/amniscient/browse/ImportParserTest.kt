package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Test
class ImportParserTest {
    @Test fun parsesBookmarks() {
        val j = """{"version":1,"source":"chrome-windows","profile":"Default","exportedAt":"t","bookmarks":[{"title":"A","url":"https://a.example/","path":["bar"],"added":1}],"history":[{"url":"https://h.example/","title":"H","lastVisit":2,"visitCount":3}]}"""
        val p = ImportParser.parse(j)
        assertEquals(1, p.bookmarks.size)
        assertEquals("https://a.example", p.bookmarks[0].url)
        assertEquals(1, p.history.size)
    }
    @Test(expected = IllegalArgumentException::class)
    fun rejectsPasswordKey() {
        ImportParser.parse("""{"version":1,"password":"no","bookmarks":[],"history":[]}""")
    }
    @Test(expected = IllegalArgumentException::class)
    fun rejectsBadVersion() {
        ImportParser.parse("""{"version":2,"bookmarks":[],"history":[]}""")
    }
    @Test fun mergeByNormalizedUrl() {
        val j = """{"version":1,"bookmarks":[{"title":"A","url":"https://A.Example/","path":[],"added":1},{"title":"B","url":"https://a.example","path":[],"added":2}],"history":[{"url":"https://H.Example/","title":"old","lastVisit":1,"visitCount":1},{"url":"https://h.example","title":"new","lastVisit":9,"visitCount":2}]}"""
        val p = ImportParser.parse(j)
        assertEquals(1, p.bookmarks.size)
        assertEquals("https://a.example", p.bookmarks[0].url)
        assertEquals(1, p.history.size)
        assertEquals("new", p.history[0].title)
        assertEquals(true, ImportParser.shouldUpdateHistory(1, 9))
        assertEquals(false, ImportParser.shouldUpdateHistory(9, 1))
    }
}
