package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
class NavResolverTest {
    @Test fun hostGetsHttps() { assertEquals("https://example.com", NavResolver.resolve("example.com")) }
    @Test fun searchGoesDdg() { assertTrue(NavResolver.resolve("hello world").startsWith("https://duckduckgo.com/?q=")) }
    @Test fun keepsHttps() { assertEquals("https://amni-scient.com/x", NavResolver.resolve("https://amni-scient.com/x")) }
}
