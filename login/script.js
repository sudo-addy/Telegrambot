document.addEventListener('DOMContentLoaded', () => {
    // Initialize Lucide Icons
    if (typeof lucide !== 'undefined') {
        lucide.createIcons();
    }

    const loginForm = document.getElementById('loginForm');
    const passwordInput = document.getElementById('password');
    const togglePasswordBtn = document.getElementById('togglePassword');
    const toggleIcon = togglePasswordBtn?.querySelector('.toggle-icon');

    // Password Visibility Toggle
    if (togglePasswordBtn && passwordInput) {
        togglePasswordBtn.addEventListener('click', () => {
            const isPassword = passwordInput.getAttribute('type') === 'password';
            
            // Toggle input type
            passwordInput.setAttribute('type', isPassword ? 'text' : 'password');
            
            // Update icon dynamically
            if (toggleIcon) {
                toggleIcon.setAttribute('data-lucide', isPassword ? 'eye-off' : 'eye');
                // Re-create icons to apply the change
                if (typeof lucide !== 'undefined') {
                    lucide.createIcons();
                }
            }
        });
    }

    // Form Submission Handling with feedback animation
    if (loginForm) {
        loginForm.addEventListener('submit', (e) => {
            e.preventDefault();
            
            const email = document.getElementById('email').value;
            const password = passwordInput.value;
            const submitBtn = loginForm.querySelector('.submit-btn');
            const submitBtnText = submitBtn.querySelector('span');
            const submitBtnIcon = submitBtn.querySelector('i');

            if (!submitBtn) return;

            // Simple loading state animation
            submitBtn.style.pointerEvents = 'none';
            submitBtn.style.opacity = '0.8';
            if (submitBtnText) submitBtnText.textContent = 'Connecting...';
            if (submitBtnIcon) {
                submitBtnIcon.setAttribute('data-lucide', 'loader-2');
                submitBtnIcon.classList.add('spinning-loader');
                if (typeof lucide !== 'undefined') {
                    lucide.createIcons();
                }
            }

            console.log('Login attempt for:', email);

            // Mock network latency
            setTimeout(() => {
                // Success state simulation
                if (submitBtnText) submitBtnText.textContent = 'Access Granted';
                submitBtn.style.background = 'linear-gradient(135deg, #10b981 0%, #059669 100%)';
                submitBtn.style.boxShadow = '0 6px 20px rgba(16, 185, 129, 0.4)';
                
                if (submitBtnIcon) {
                    submitBtnIcon.setAttribute('data-lucide', 'check-circle-2');
                    submitBtnIcon.classList.remove('spinning-loader');
                    if (typeof lucide !== 'undefined') {
                        lucide.createIcons();
                    }
                }
                
                // Clear input fields
                setTimeout(() => {
                    alert(`Welcome! Login successful for user: ${email}`);
                    // Reset button
                    submitBtn.style.pointerEvents = 'auto';
                    submitBtn.style.opacity = '1';
                    submitBtn.style.background = '';
                    submitBtn.style.boxShadow = '';
                    if (submitBtnText) submitBtnText.textContent = 'Sign In';
                    if (submitBtnIcon) {
                        submitBtnIcon.setAttribute('data-lucide', 'arrow-right');
                        if (typeof lucide !== 'undefined') {
                            lucide.createIcons();
                        }
                    }
                }, 400);

            }, 1800);
        });
    }
});
