package com.amniscient.browse
object CookieHosts {
    fun names(header: String?): List<String> {
        if (header.isNullOrBlank()) return emptyList()
        return header.split(';').map { it.substringBefore('=').trim() }.filter { it.isNotEmpty() }
    }
    fun expirePair(name: String): String = "$name=; Max-Age=0; Path=/"
    fun acceptCookies(host: String, blocked: Set<String>): Boolean = host.isNotEmpty() && host !in blocked
}
