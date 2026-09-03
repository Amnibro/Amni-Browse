package com.amniscient.browse

import android.animation.ValueAnimator
import android.content.Context
import android.graphics.Canvas
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Shader
import android.util.AttributeSet
import android.view.View
import android.view.animation.LinearInterpolator
import androidx.core.graphics.ColorUtils

/** Accent load strip: determinate fill plus a sliding shimmer while pages load. */
class LoadPulseBar @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
) : View(context, attrs) {
    var accentColor = 0
    var progress = 0f
        set(value) {
            field = value.coerceIn(0f, 1f)
            invalidate()
        }
    var active = false
        set(value) {
            field = value
            if (value) startShimmer() else stopShimmer()
            invalidate()
        }

    private val trackPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val fillPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val shimmerPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private var shimmer = 0f
    private var shimmerAnim: ValueAnimator? = null

    init {
        setWillNotDraw(false)
    }

    override fun onDraw(canvas: Canvas) {
        if (!active && progress <= 0f) return
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) return
        trackPaint.color = ColorUtils.setAlphaComponent(accentColor, 36)
        canvas.drawRect(0f, 0f, w, h, trackPaint)
        if (progress > 0f) {
            fillPaint.color = accentColor
            canvas.drawRect(0f, 0f, w * progress, h, fillPaint)
        }
        if (active) {
            val band = w * 0.35f
            val x = shimmer * (w + band) - band
            shimmerPaint.shader = LinearGradient(
                x, 0f, x + band, 0f,
                intArrayOf(
                    ColorUtils.setAlphaComponent(accentColor, 0),
                    ColorUtils.setAlphaComponent(accentColor, 220),
                    ColorUtils.setAlphaComponent(accentColor, 0),
                ),
                floatArrayOf(0f, 0.5f, 1f),
                Shader.TileMode.CLAMP,
            )
            canvas.drawRect(0f, 0f, w, h, shimmerPaint)
        }
    }

    private fun startShimmer() {
        if (shimmerAnim?.isRunning == true) return
        shimmerAnim = ValueAnimator.ofFloat(0f, 1f).apply {
            duration = 1100L
            repeatCount = ValueAnimator.INFINITE
            interpolator = LinearInterpolator()
            addUpdateListener {
                shimmer = it.animatedValue as Float
                invalidate()
            }
            start()
        }
    }

    private fun stopShimmer() {
        shimmerAnim?.cancel()
        shimmerAnim = null
        shimmer = 0f
    }

    override fun onDetachedFromWindow() {
        stopShimmer()
        super.onDetachedFromWindow()
    }
}
