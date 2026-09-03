package com.amniscient.browse
import android.content.Context
import androidx.room.Dao
import androidx.room.Database
import androidx.room.Entity
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.PrimaryKey
import androidx.room.Query
import androidx.room.Room
import androidx.room.RoomDatabase
@Entity(tableName = "bookmarks")
data class BookmarkEntity(@PrimaryKey val url: String, val title: String, val path: String, val added: Long)
@Entity(tableName = "history")
data class HistoryEntity(@PrimaryKey val url: String, val title: String, val lastVisit: Long, val visitCount: Int)
@Dao
interface StoreDao {
    @Query("SELECT * FROM bookmarks ORDER BY title")
    suspend fun allBookmarks(): List<BookmarkEntity>
    @Query("SELECT * FROM history ORDER BY lastVisit DESC LIMIT 200")
    suspend fun recentHistory(): List<HistoryEntity>
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertBookmark(row: BookmarkEntity): Long
    @Query("UPDATE history SET title = :title, lastVisit = :lastVisit, visitCount = visitCount + :inc WHERE url = :url")
    suspend fun touchHistory(url: String, title: String, lastVisit: Long, inc: Int): Int
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    suspend fun insertHistory(row: HistoryEntity): Long
    @Query("SELECT lastVisit FROM history WHERE url = :url")
    suspend fun historyLast(url: String): Long?
    @Query("SELECT * FROM history WHERE url LIKE '%'||:q||'%' OR title LIKE '%'||:q||'%' ORDER BY visitCount DESC, lastVisit DESC LIMIT :n")
    suspend fun searchHistory(q: String, n: Int): List<HistoryEntity>
    @Query("SELECT * FROM bookmarks WHERE url LIKE '%'||:q||'%' OR title LIKE '%'||:q||'%' ORDER BY added DESC LIMIT :n")
    suspend fun searchBookmarks(q: String, n: Int): List<BookmarkEntity>
    @Query("DELETE FROM history")
    suspend fun clearHistory(): Int
    @Query("DELETE FROM bookmarks WHERE url = :url")
    suspend fun deleteBookmark(url: String): Int
    @Query("SELECT * FROM bookmarks ORDER BY added DESC LIMIT :n")
    suspend fun topBookmarks(n: Int): List<BookmarkEntity>
    @Query("UPDATE bookmarks SET title = :title, path = :path WHERE url = :url")
    suspend fun updateBookmark(url: String, title: String, path: String): Int
}
@Database(entities = [BookmarkEntity::class, HistoryEntity::class], version = 1, exportSchema = false)
abstract class AppDb : RoomDatabase() {
    abstract fun dao(): StoreDao
    companion object {
        @Volatile private var inst: AppDb? = null
        fun get(ctx: Context): AppDb = inst ?: synchronized(this) {
            inst ?: Room.databaseBuilder(ctx.applicationContext, AppDb::class.java, "amni.db").build().also { inst = it }
        }
    }
}
