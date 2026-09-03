package com.amniscient.browse

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class BrowserPoliciesTest {
    @Test fun failedMainFrameFinishDoesNotClearFailure() {
        val state = MainFrameNavigationState()
        state.onPageStarted("https://offline.test/")
        state.onMainFrameError("https://offline.test/")

        assertFalse(state.onPageFinished("https://offline.test/"))
        assertTrue(state.hasFailure())
        assertEquals("https://offline.test/", state.failedUrl)
    }

    @Test fun retryOnlyClearsAfterVerifiedSuccessfulFinish() {
        val state = MainFrameNavigationState()
        state.onPageStarted("https://offline.test/")
        state.onMainFrameError("https://offline.test/")

        state.onPageStarted("https://offline.test/")
        assertTrue("failure remains visible while retry is loading", state.hasFailure())
        assertFalse("a stale finish cannot clear the retry UI", state.onPageFinished("https://other.test/"))
        assertTrue(state.hasFailure())
        assertTrue(state.onPageFinished("https://offline.test/"))
        assertFalse(state.hasFailure())
        assertNull(state.failedUrl)
    }

    @Test fun successfulNavigationWithoutPriorFailureFinishesCleanly() {
        val state = MainFrameNavigationState()
        state.onPageStarted("https://example.test/")
        assertTrue(state.onPageFinished("https://example.test/"))
        assertFalse(state.hasFailure())
    }

    @Test fun reportsHttpAsInsecureWithoutBlockingIt() {
        assertEquals(ConnectionSecurity.INSECURE_HTTP, BrowserPolicies.connectionSecurity("http://192.168.1.1/"))
        assertEquals(ConnectionSecurity.INSECURE_HTTP, BrowserPolicies.connectionSecurity("http://router.local/"))
        assertEquals(ConnectionSecurity.SECURE, BrowserPolicies.connectionSecurity("https://example.com"))
        assertEquals(ConnectionSecurity.LOCAL_OR_INTERNAL, BrowserPolicies.connectionSecurity("about:blank"))
    }

    @Test fun externalIntentPolicyAllowsExpectedSchemesOnly() {
        assertTrue(BrowserPolicies.canOpenExternally("mailto:hello@example.com"))
        assertTrue(BrowserPolicies.canOpenExternally("tel:+15555550100"))
        assertTrue(BrowserPolicies.canOpenExternally("intent://scan/#Intent;scheme=zxing;end"))
        assertFalse(BrowserPolicies.canOpenExternally("javascript:alert(1)"))
        assertFalse(BrowserPolicies.canOpenExternally("custom-unknown:payload"))
    }

    @Test fun downloadPolicyAcceptsNetworkFilesOnly() {
        assertTrue(BrowserPolicies.canDownload("https://example.com/file.pdf"))
        assertTrue(BrowserPolicies.canDownload("http://router.local/config.bin"))
        assertFalse(BrowserPolicies.canDownload("blob:https://example.com/id"))
        assertFalse(BrowserPolicies.canDownload("file:///sdcard/secret"))
    }

    @Test fun filePickerTypesAreSanitizedAndDefaulted() {
        assertArrayEquals(arrayOf("*/*"), BrowserPolicies.fileChooserAcceptTypes(emptyArray()))
        assertArrayEquals(
            arrayOf("image/*", "application/pdf"),
            BrowserPolicies.fileChooserAcceptTypes(arrayOf(" image/* ", "", "image/*", "application/pdf")),
        )
    }

    @Test fun backForwardReloadRespectCurrentSurface() {
        assertEquals(
            BrowserControlResult.GO_BACK,
            BrowserPolicies.controlResult(BrowserControl.BACK, homeVisible = false, canGoBack = true, canGoForward = false),
        )
        assertEquals(
            BrowserControlResult.GO_FORWARD,
            BrowserPolicies.controlResult(BrowserControl.FORWARD, homeVisible = false, canGoBack = false, canGoForward = true),
        )
        assertEquals(
            BrowserControlResult.RELOAD,
            BrowserPolicies.controlResult(BrowserControl.RELOAD, homeVisible = false, canGoBack = false, canGoForward = false),
        )
        assertEquals(
            BrowserControlResult.NONE,
            BrowserPolicies.controlResult(BrowserControl.RELOAD, homeVisible = true, canGoBack = true, canGoForward = true),
        )
    }

    @Test fun dangerousSchemesNeverLoadInWebView() {
        assertTrue(BrowserPolicies.isDangerousScheme("javascript:alert(1)"))
        assertTrue(BrowserPolicies.isDangerousScheme("file:///data/data/com.amniscient.browse/files/x"))
        assertTrue(BrowserPolicies.isDangerousScheme("content://com.evil/secret"))
        assertFalse(BrowserPolicies.canLoadInWebView("javascript:alert(1)"))
        assertFalse(BrowserPolicies.canLoadInWebView("file:///sdcard/x"))
        assertTrue(BrowserPolicies.canLoadInWebView("https://example.com/"))
        assertTrue(BrowserPolicies.canLoadInWebView("http://192.168.1.1/"))
    }

    @Test fun silenceScriptKillsMediaElements() {
        assertTrue(BrowserPolicies.SILENCE_JS.contains("pause()"))
        assertTrue(BrowserPolicies.SILENCE_JS.contains("video,audio"))
        assertTrue(BrowserPolicies.SILENCE_JS.contains("speechSynthesis"))
    }
}
