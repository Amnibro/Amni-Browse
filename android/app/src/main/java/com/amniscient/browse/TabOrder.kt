package com.amniscient.browse
object TabOrder {
    fun <T> move(items: MutableList<T>, from: Int, to: Int, active: Int): Int {
        if (from == to || from !in items.indices || to !in items.indices) return active
        val item = items.removeAt(from)
        items.add(to, item)
        return remap(active, from, to)
    }
    fun remap(active: Int, from: Int, to: Int): Int = when {
        active == from -> to
        from < active && to >= active -> active - 1
        from > active && to <= active -> active + 1
        else -> active
    }
}
