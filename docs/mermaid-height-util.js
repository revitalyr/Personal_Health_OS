class MermaidHeightAdjuster {
    constructor() {
        this.currentScale = 1.0;
    }

    setupScaleControls() {
        document.addEventListener('click', (e) => {
            const btn = e.target.closest('[data-scale-delta]');
            if (btn) {
                const delta = parseFloat(btn.getAttribute('data-scale-delta'));
                this.changeScale(delta);
                return;
            }
            const resetBtn = e.target.closest('[data-reset-scale]');
            if (resetBtn) {
                this.resetScale();
            }
        });
    }

    changeScale(delta) {
        const newScale = Math.min(3.0, Math.max(0.3, this.currentScale + delta));
        this.currentScale = Math.round(newScale * 100) / 100;
        this.applyScale();
        this.updateScaleDisplay();
    }

    resetScale() {
        this.currentScale = 1.0;
        this.applyScale();
        this.updateScaleDisplay();
    }

    applyScale() {
        document.querySelectorAll('.diagram-container').forEach(container => {
            container.style.transform = `scale(${this.currentScale})`;
            container.style.transformOrigin = 'top left';
        });
    }

    updateScaleDisplay() {
        document.querySelectorAll('.scale-display').forEach(el => {
            el.textContent = Math.round(this.currentScale * 100) + '%';
        });
    }

    init() {
        this.setupScaleControls();
        this.updateScaleDisplay();
    }
}
