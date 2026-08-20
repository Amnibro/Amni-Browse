package com.amniscient.browse
import android.content.Context
import org.json.JSONArray
import org.json.JSONObject
data class Tab(var url: String, var title: String, var home: Boolean)
object SessionStore {
    private const val PREF = "amni_session"
    fun save(ctx: Context, tabs: List<Tab>, index: Int) {
        val arr = JSONArray()
        for (t in tabs) {
            val o = JSONObject()
            o.put("url", t.url)
            o.put("title", t.title)
            o.put("home", t.home)
            arr.put(o)
        }
        ctx.getSharedPreferences(PREF, Context.MODE_PRIVATE).edit()
            .putString("tabs", arr.toString())
            .putInt("index", index)
            .apply()
    }
    fun load(ctx: Context): Pair<List<Tab>, Int>? {
        val p = ctx.getSharedPreferences(PREF, Context.MODE_PRIVATE)
        val raw = p.getString("tabs", null) ?: return null
        val arr = JSONArray(raw)
        if (arr.length() == 0) return null
        val tabs = mutableListOf<Tab>()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            tabs.add(Tab(o.optString("url"), o.optString("title", "Tab"), o.optBoolean("home", o.optString("url").isEmpty())))
        }
        val idx = p.getInt("index", 0).coerceIn(0, tabs.lastIndex)
        return tabs to idx
    }
}
