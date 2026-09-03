package com.amniscient.browse

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SessionCodecTest {
    @Test fun openTabsRoundTripWithIndexRemap() {
        val tabs = listOf(
            Tab("https://a", "A", false, priv = true),
            Tab("https://b", "B", false, priv = false),
            Tab("https://c", "C", false, priv = false),
        )
        val enc = SessionCodec.encode(tabs, 2)
        val (out, idx) = SessionCodec.decode(enc.first, enc.second)
        assertEquals(3, out.size)
        assertTrue(out[0].priv)
        assertEquals("", out[0].url)
        assertEquals("https://b", out[1].url)
        assertEquals("https://c", out[2].url)
        assertEquals(2, idx)
    }

    @Test fun allPrivateKeepsTabCountButRedactsUrls() {
        val tabs = listOf(
            Tab("https://secret", "Secret", false, priv = true),
            Tab("", "Private", true, priv = true),
        )
        val enc = SessionCodec.encode(tabs, 1)
        val (out, idx) = SessionCodec.decode(enc.first, enc.second)
        assertEquals(2, out.size)
        assertTrue(out.all { it.priv })
        assertTrue(out.all { it.url.isEmpty() })
        assertEquals(1, idx)
    }

    @Test fun defaultNewTabIsOpen() {
        assertEquals(false, Tab("", "Home", true).priv)
    }

    @Test fun processRecreationRoundTripsMixedTabs() {
        val original = listOf(
            Tab("https://private.example", "Private", false, priv = true, group = "Hidden"),
            Tab("https://public.example", "Public", false, priv = false, group = "Work"),
            Tab("", "Home", true, priv = false),
        )
        val encoded = SessionCodec.encode(original, 1)
        val restored = SessionCodec.decodeOrNull(encoded.first, encoded.second)!!

        assertEquals(3, restored.first.size)
        assertTrue(restored.first[0].priv)
        assertEquals("", restored.first[0].url)
        assertEquals("https://public.example", restored.first[1].url)
        assertEquals("Work", restored.first[1].group)
        assertEquals(1, restored.second)
    }

    @Test fun corruptProcessStateFallsBackInsteadOfCrashing() {
        assertNull(SessionCodec.decodeOrNull("{not-json", 4))
    }
}
