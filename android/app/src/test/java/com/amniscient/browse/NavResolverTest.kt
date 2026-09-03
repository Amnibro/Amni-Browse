package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
class NavResolverTest {
    @Test fun hostGetsHttps() { assertEquals("https://example.com", NavResolver.resolve("example.com")) }
    @Test fun searchGoesDdg() { assertTrue(NavResolver.resolve("hello world").startsWith("https://duckduckgo.com/?q=")) }
    @Test fun keepsHttps() { assertEquals("https://amni-scient.com/x", NavResolver.resolve("https://amni-scient.com/x")) }
    @Test fun searchUsesPickedEngine() {
        assertTrue(NavResolver.resolve("hello world", "google").startsWith("https://www.google.com/search?q="))
    }
    @Test fun stripsTrackersOnResolve() {
        assertEquals("https://example.com/p", NavResolver.resolve("https://example.com/p?utm_source=x&fbclid=1"))
    }
    @Test fun keepsRealQueryWhileStripping() {
        assertEquals("https://example.com/p?q=hi", NavResolver.stripTrackers("https://example.com/p?q=hi&gclid=abc"))
    }
}
