package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
class SearchEngineTest {
    @Test fun ddgSearchUrlEncodesQuery() {
        assertEquals("https://duckduckgo.com/?q=hello+world", SearchEngine.searchUrl("ddg", "hello world"))
    }
    @Test fun unknownIdFallsBackToDdg() {
        assertTrue(SearchEngine.searchUrl("nope", "x").startsWith("https://duckduckgo.com/?q="))
    }
    @Test fun parseDdgPhrases() {
        val body = """[{"phrase":"firefox"},{"phrase":"firefox download"}]"""
        assertEquals(listOf("firefox", "firefox download"), SearchEngine.parseSuggest("ddg", body))
    }
    @Test fun parseGoogleFirefoxClient() {
        val body = """["fir",["firefox","firestick"]]"""
        assertEquals(listOf("firefox", "firestick"), SearchEngine.parseSuggest("google", body))
    }
    @Test fun parseWikipediaOpensearch() {
        val body = """["servo",["Servo","Servo motor"]]"""
        assertEquals(listOf("Servo", "Servo motor"), SearchEngine.parseSuggest("wikipedia", body))
    }
    @Test fun emptySuggestBodyIsEmpty() {
        assertEquals(emptyList<String>(), SearchEngine.parseSuggest("ddg", ""))
    }
}
