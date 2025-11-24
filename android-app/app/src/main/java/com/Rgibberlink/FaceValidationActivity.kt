package com.Rgibberlink

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.View
import android.widget.*
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.camera.core.*
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.content.ContextCompat
import com.google.mlkit.vision.common.InputImage
import com.google.mlkit.vision.face.Face
import com.google.mlkit.vision.face.FaceDetection
import com.google.mlkit.vision.face.FaceDetector
import com.google.mlkit.vision.face.FaceDetectorOptions
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

class FaceValidationActivity : AppCompatActivity() {

    companion object {
        private const val TAG = "FaceValidation"
        private const val PREF_FACE_VALIDATION_ENABLED = "face_validation_enabled"
        private const val PREF_FACE_VALIDATION_TIMEOUT = "face_validation_timeout"
        private const val DEFAULT_TIMEOUT_MS = 30000L // 30 seconds
        private const val VALIDATION_SUCCESS_DELAY_MS = 2000L // 2 seconds of continuous detection
    }

    // UI Components
    private lateinit var previewView: PreviewView
    private lateinit var statusText: TextView
    private lateinit var progressBar: ProgressBar
    private lateinit var cancelButton: Button
    private lateinit var faceOverlay: ImageView

    // Camera components
    private lateinit var cameraExecutor: ExecutorService
    private var camera: Camera? = null
    private var imageAnalyzer: ImageAnalysis? = null

    // Face detection
    private lateinit var faceDetector: FaceDetector
    private var isValidating = false
    private var faceDetected = false
    private var validationStartTime = 0L
    private val handler = Handler(Looper.getMainLooper())

    // Timeout handling
    private var timeoutRunnable: Runnable? = null

    // Preferences
    private lateinit var prefs: android.content.SharedPreferences

    // Camera permission
    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted ->
        if (isGranted) {
            startCamera()
        } else {
            showPermissionDenied()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_face_validation)

        prefs = getSharedPreferences("gibberlink_app", Context.MODE_PRIVATE)

        // Initialize face detector with real-time options
        val options = FaceDetectorOptions.Builder()
            .setPerformanceMode(FaceDetectorOptions.PERFORMANCE_MODE_FAST)
            .setContourMode(FaceDetectorOptions.CONTOUR_MODE_NONE)
            .setClassificationMode(FaceDetectorOptions.CLASSIFICATION_MODE_NONE)
            .setMinFaceSize(0.15f)
            .enableTracking()
            .build()

        faceDetector = FaceDetection.getClient(options)

        initializeViews()
        setupCameraExecutor()

        // Check camera permission
        if (hasCameraPermission()) {
            startCamera()
        } else {
            requestPermissionLauncher.launch(Manifest.permission.CAMERA)
        }

        // Set timeout
        val timeout = prefs.getLong(PREF_FACE_VALIDATION_TIMEOUT, DEFAULT_TIMEOUT_MS)
        startTimeout(timeout)
    }

    private fun initializeViews() {
        previewView = findViewById(R.id.previewView)
        statusText = findViewById(R.id.statusText)
        progressBar = findViewById(R.id.progressBar)
        cancelButton = findViewById(R.id.cancelButton)
        faceOverlay = findViewById(R.id.faceOverlay)

        cancelButton.setOnClickListener { cancelValidation() }

        updateStatus("Initializing face validation...", false)
    }

    private fun setupCameraExecutor() {
        cameraExecutor = Executors.newSingleThreadExecutor()
    }

    private fun hasCameraPermission(): Boolean {
        return ContextCompat.checkSelfPermission(
            this, Manifest.permission.CAMERA
        ) == PackageManager.PERMISSION_GRANTED
    }

    private fun showPermissionDenied() {
        updateStatus("Camera permission required for face validation", false)
        Toast.makeText(this, "Camera permission is required", Toast.LENGTH_LONG).show()
        finishWithResult(false)
    }

    private fun startCamera() {
        updateStatus("Starting camera...", false)

        val cameraProviderFuture = ProcessCameraProvider.getInstance(this)

        cameraProviderFuture.addListener({
            try {
                val cameraProvider: ProcessCameraProvider = cameraProviderFuture.get()

                // Preview
                val preview = Preview.Builder()
                    .build()
                    .also {
                        it.setSurfaceProvider(previewView.surfaceProvider)
                    }

                // Image analysis
                imageAnalyzer = ImageAnalysis.Builder()
                    .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                    .build()
                    .also {
                        it.setAnalyzer(cameraExecutor) { imageProxy ->
                            processImage(imageProxy)
                        }
                    }

                // Select back camera
                val cameraSelector = CameraSelector.DEFAULT_FRONT_CAMERA

                try {
                    // Unbind use cases before rebinding
                    cameraProvider.unbindAll()

                    // Bind use cases to camera
                    camera = cameraProvider.bindToLifecycle(
                        this, cameraSelector, preview, imageAnalyzer
                    )

                    updateStatus("Face validation active - please look at the camera", false)

                } catch (exc: Exception) {
                    Log.e(TAG, "Use case binding failed", exc)
                    updateStatus("Camera initialization failed", false)
                    finishWithResult(false)
                }

            } catch (exc: Exception) {
                Log.e(TAG, "Camera provider failed", exc)
                updateStatus("Camera not available", false)
                finishWithResult(false)
            }
        }, ContextCompat.getMainExecutor(this))
    }

    @androidx.annotation.OptIn(androidx.camera.core.ExperimentalGetImage::class)
    private fun processImage(imageProxy: ImageProxy) {
        val mediaImage = imageProxy.image
        if (mediaImage != null) {
            val image = InputImage.fromMediaImage(mediaImage, imageProxy.imageInfo.rotationDegrees)

            faceDetector.process(image)
                .addOnSuccessListener { faces ->
                    onFacesDetected(faces)
                }
                .addOnFailureListener { e ->
                    Log.e(TAG, "Face detection failed", e)
                }
        }

        imageProxy.close()
    }

    private fun onFacesDetected(faces: List<Face>) {
        runOnUiThread {
            if (faces.isNotEmpty()) {
                onFaceDetected(faces.first())
            } else {
                onNoFaceDetected()
            }
        }
    }

    private fun onFaceDetected(face: Face) {
        if (!faceDetected) {
            faceDetected = true
            validationStartTime = System.currentTimeMillis()
            updateStatus("Face detected - maintaining eye contact...", true)
            showFaceOverlay(true)
        }

        // Check if face has been continuously detected for the required time
        val continuousDetectionTime = System.currentTimeMillis() - validationStartTime
        if (continuousDetectionTime >= VALIDATION_SUCCESS_DELAY_MS) {
            onValidationSuccess()
        }
    }

    private fun onNoFaceDetected() {
        if (faceDetected) {
            faceDetected = false
            validationStartTime = 0L
            updateStatus("Face validation active - please look at the camera", false)
            showFaceOverlay(false)
        }
    }

    private fun showFaceOverlay(show: Boolean) {
        faceOverlay.visibility = if (show) View.VISIBLE else View.GONE
    }

    private fun onValidationSuccess() {
        updateStatus("Human validation successful!", false)
        showFaceOverlay(false)
        cancelTimeout()

        progressBar.visibility = View.VISIBLE
        statusText.text = "Validation complete - proceeding..."

        // Brief success animation
        handler.postDelayed({
            finishWithResult(true)
        }, 1000)
    }

    private fun updateStatus(message: String, showProgress: Boolean) {
        statusText.text = message
        progressBar.visibility = if (showProgress) View.VISIBLE else View.GONE
    }

    private fun startTimeout(timeoutMs: Long) {
        timeoutRunnable = Runnable {
            updateStatus("Validation timeout - please try again", false)
            finishWithResult(false)
        }
        handler.postDelayed(timeoutRunnable!!, timeoutMs)
    }

    private fun cancelTimeout() {
        timeoutRunnable?.let { handler.removeCallbacks(it) }
        timeoutRunnable = null
    }

    private fun cancelValidation() {
        cancelTimeout()
        finishWithResult(false)
    }

    private fun finishWithResult(success: Boolean) {
        val result = Intent().apply {
            putExtra("face_validation_success", success)
        }
        setResult(if (success) RESULT_OK else RESULT_CANCELED, result)
        finish()
    }

    override fun onDestroy() {
        super.onDestroy()
        cameraExecutor.shutdown()
        faceDetector.close()
        cancelTimeout()
    }
}
