package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Test
class CookieHostsTest {
    @Test fun splitsCookieHeaderNames() {
        assertEquals(listOf("sid", "pref"), CookieHosts.names("sid=abc; pref=1"))
    }
    @Test fun expirePairZerosMaxAge() {
        assertEquals("sid=; Max-Age=0; Path=/", CookieHosts.expirePair("sid"))
    }
    @Test fun emptyHeaderIsEmpty() {
        assertEquals(emptyList<String>(), CookieHosts.names(""))
        assertEquals(emptyList<String>(), CookieHosts.names(null))
    }
    @Test fun acceptCookiesFalseWhenHostBlocked() {
        assertEquals(false, CookieHosts.acceptCookies("ads.example", setOf("ads.example")))
    }
    @Test fun acceptCookiesTrueWhenHostOpen() {
        assertEquals(true, CookieHosts.acceptCookies("news.example", setOf("ads.example")))
    }
    @Test fun acceptCookiesFalseWhenHostEmpty() {
        assertEquals(false, CookieHosts.acceptCookies("", setOf()))
    }
}
