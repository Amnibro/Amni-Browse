package com.amniscient.browse

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.util.AttributeSet
import android.widget.FrameLayout

/** Tab strip + toolbar shell; draws the active folder-tab outline without affecting height. */
class ChromeShellLayout @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
) : FrameLayout(context, attrs) {
    var tabLeft = 0f
    var tabRight = 0f
    var tabTop = 0f
    var bodyTop = 0f
    var borderColor = 0
    var strokePx = 1f
    var cornerPx = 10f
    var outlineVisible = false
        set(value) {
            field = value
            invalidate()
        }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE }
    private val path = Path()
    private val arc = RectF()

    init {
        setWillNotDraw(false)
    }

    override fun dispatchDraw(canvas: Canvas) {
        super.dispatchDraw(canvas)
        if (!outlineVisible || tabRight <= tabLeft || bodyTop <= tabTop) return
        paint.color = borderColor
        paint.strokeWidth = strokePx
        val half = strokePx / 2f
        val w = width.toFloat()
        val h = height.toFloat()
        val r = cornerPx.coerceAtMost((tabRight - tabLeft) / 2f - half)
        path.reset()
        path.moveTo(half, h - half)
        path.lineTo(w - half, h - half)
        path.lineTo(w - half, bodyTop + half)
        path.lineTo(tabRight - half, bodyTop + half)
        path.lineTo(tabRight - half, tabTop + r)
        arc.set(tabRight - 2f * r, tabTop + half, tabRight - half, tabTop + 2f * r)
        path.arcTo(arc, 0f, -90f, false)
        path.lineTo(tabLeft + r, tabTop + half)
        arc.set(tabLeft + half, tabTop + half, tabLeft + 2f * r, tabTop + 2f * r)
        path.arcTo(arc, 270f, -90f, false)
        path.lineTo(tabLeft + half, bodyTop + half)
        path.lineTo(half, bodyTop + half)
        path.lineTo(half, h - half)
        canvas.drawPath(path, paint)
    }
}
