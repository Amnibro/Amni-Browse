package com.amniscient.browse
import org.junit.Assert.assertEquals
import org.junit.Test
class TabOrderTest {
    @Test fun moveForwardShiftsActive() {
        val tabs = mutableListOf("a", "b", "c", "d")
        val active = TabOrder.move(tabs, 0, 3, 0)
        assertEquals(listOf("b", "c", "d", "a"), tabs)
        assertEquals(3, active)
    }
    @Test fun moveBackwardRemapsActive() {
        val tabs = mutableListOf("a", "b", "c")
        val active = TabOrder.move(tabs, 2, 0, 2)
        assertEquals(listOf("c", "a", "b"), tabs)
        assertEquals(0, active)
    }
    @Test fun movingOtherTabLeavesActive() {
        val tabs = mutableListOf("a", "b", "c")
        val active = TabOrder.move(tabs, 0, 1, 2)
        assertEquals(listOf("b", "a", "c"), tabs)
        assertEquals(2, active)
    }
    @Test fun sameIndexIsNoop() {
        val tabs = mutableListOf("a", "b")
        val active = TabOrder.move(tabs, 1, 1, 1)
        assertEquals(listOf("a", "b"), tabs)
        assertEquals(1, active)
    }
}
