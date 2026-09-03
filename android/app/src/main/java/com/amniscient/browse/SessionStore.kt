package com.amniscient.browse

import android.content.Context
import android.graphics.Bitmap
import org.json.JSONArray
import org.json.JSONObject

data class Tab(
    var url: String,
    var title: String,
    var home: Boolean,
    var priv: Boolean = false,
    var thumb: Bitmap? = null,
    var icon: Bitmap? = null,
    var group: String? = null,
)

object SessionCodec {
    /** Persist tab strip layout; open tabs keep URLs, private tabs redact URLs on disk. */
    fun encode(tabs: List<Tab>, index: Int): Pair<String, Int> {
        val arr = JSONArray()
        for (t in tabs) {
            val o = JSONObject()
            o.put("priv", t.priv)
            if (t.priv) {
                o.put("url", "")
                o.put("title", "Private")
                o.put("home", true)
            } else {
                o.put("url", t.url)
                o.put("title", t.title)
                o.put("home", t.home)
                t.group?.takeIf { it.isNotBlank() }?.let { o.put("group", it) }
            }
            arr.put(o)
        }
        val idx = index.coerceIn(0, tabs.lastIndex.coerceAtLeast(0))
        return arr.toString() to idx
    }

    fun decode(raw: String, index: Int): Pair<List<Tab>, Int> {
        val arr = JSONArray(raw)
        val tabs = mutableListOf<Tab>()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            val priv = o.optBoolean("priv", false)
            tabs.add(
                Tab(
                    url = if (priv) "" else o.optString("url"),
                    title = if (priv) "Private" else o.optString("title", "Tab"),
                    home = if (priv) true else o.optBoolean("home", o.optString("url").isEmpty()),
                    priv = priv,
                    group = if (priv) null else o.optString("group").takeIf { it.isNotBlank() },
                ),
            )
        }
        if (tabs.isEmpty()) return emptyList<Tab>() to 0
        return tabs to index.coerceIn(0, tabs.lastIndex)
    }

    fun decodeOrNull(raw: String, index: Int): Pair<List<Tab>, Int>? =
        try {
            decode(raw, index)
        } catch (_: Exception) {
            null
        }
}

object SessionStore {
    private const val PREF = "amni_session"
    private const val KEY_TABS = "tabs"
    private const val KEY_INDEX = "index"

    fun save(ctx: Context, tabs: List<Tab>, index: Int) {
        if (tabs.isEmpty()) return
        val enc = SessionCodec.encode(tabs, index)
        ctx.getSharedPreferences(PREF, Context.MODE_PRIVATE).edit()
            .putString(KEY_TABS, enc.first)
            .putInt(KEY_INDEX, enc.second)
            .apply()
    }

    fun load(ctx: Context): Pair<List<Tab>, Int>? {
        val p = ctx.getSharedPreferences(PREF, Context.MODE_PRIVATE)
        val raw = p.getString(KEY_TABS, null) ?: return null
        val (tabs, idx) = SessionCodec.decodeOrNull(raw, p.getInt(KEY_INDEX, 0)) ?: return null
        if (tabs.isEmpty()) return null
        return tabs to idx
    }
}
