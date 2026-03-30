/**
 * Mermaid Diagram Height Utility
 * Automatically adjusts diagram container heights based on SVG content
 */

class MermaidHeightAdjuster {
    constructor() {
        this.currentScale = 1.0;
        this.activeTabId = 'overview';
        this.observer = null;
        this.init();
    }

    // Auto-resize mermaid containers based on SVG natural height
    adjustDiagramHeight(container) {
        if (!container) return;
        const svg = container.querySelector('.mermaid svg');
        if (!svg) return;
        
        // Get natural SVG dimensions
        const bbox = svg.getBBox ? svg.getBBox() : { width: 800, height: 400 };
        const svgHeight = Math.max(200, bbox.height + 40);
        
        // Set wrapper to auto height
        const wrapper = container.querySelector('.mermaid-wrapper');
        if (wrapper) {
            wrapper.style.height = 'auto';
            wrapper.style.minHeight = 'auto';
        }
        
        // Ensure SVG has no fixed height constraints
        svg.style.height = 'auto';
        svg.style.maxHeight = 'none';
        svg.removeAttribute('height');
        
        console.log(`Adjusted height for diagram: ${svgHeight}px`);
    }

    // Apply scale transform to active diagram
    applyScale(scale) {
        const activePane = document.querySelector('.tab-pane.active');
        if (!activePane) return;
        
        const mermaidElements = activePane.querySelectorAll('.mermaid');
        mermaidElements.forEach(mermaidElem => {
            // Reset any existing transform
            mermaidElem.style.transform = `scale(${scale})`;
            mermaidElem.style.transformOrigin = 'center top';
            // Ensure container can expand
            const wrapper = mermaidElem.closest('.mermaid-wrapper');
            if (wrapper) {
                wrapper.style.overflow = 'visible';
                wrapper.style.height = 'auto';
            }
        });
        
        // Update display
        const scaleDisplay = activePane.querySelector('.scale-display');
        if (scaleDisplay) {
            scaleDisplay.textContent = Math.round(scale * 100) + '%';
        }
    }

    // Reset scale and re-fit
    resetScale() {
        this.currentScale = 1.0;
        this.applyScale(this.currentScale);
        // After reset, re-adjust heights
        setTimeout(() => {
            const activePane = document.querySelector('.tab-pane.active');
            if (activePane) {
                const containers = activePane.querySelectorAll('.diagram-container');
                containers.forEach(container => this.adjustDiagramHeight(container));
            }
        }, 50);
    }

    changeScale(delta) {
        this.currentScale = Math.max(0.5, Math.min(2.0, this.currentScale + delta));
        this.applyScale(this.currentScale);
    }

    // Initialize mermaid for all diagrams and adjust heights
    async initMermaidInTab(tabElement) {
        const mermaidDivs = tabElement.querySelectorAll('.mermaid');
        if (mermaidDivs.length === 0) return;
        
        // Run mermaid on all diagrams in this tab
        try {
            await mermaid.run({
                nodes: mermaidDivs,
                suppressErrors: true,
            });
            
            // After rendering, adjust heights
            setTimeout(() => {
                const containers = tabElement.querySelectorAll('.diagram-container');
                containers.forEach(container => this.adjustDiagramHeight(container));
                
                // Apply current scale if any
                if (this.currentScale !== 1.0) {
                    this.applyScale(this.currentScale);
                }
            }, 100);
        } catch (err) {
            console.warn("Mermaid render error:", err);
        }
    }

    // Switch tabs
    async showTab(tabId) {
        // Hide all panes
        const allPanes = document.querySelectorAll('.tab-pane');
        allPanes.forEach(pane => pane.classList.remove('active'));
        
        // Show selected pane
        const targetPane = document.getElementById(tabId);
        if (targetPane) {
            targetPane.classList.add('active');
            this.activeTabId = tabId;
            
            // Update active button
            const buttons = document.querySelectorAll('.nav-tab');
            buttons.forEach(btn => {
                if (btn.getAttribute('data-tab') === tabId) {
                    btn.classList.add('active');
                } else {
                    btn.classList.remove('active');
                }
            });
            
            // Reset scale for new tab
            this.currentScale = 1.0;
            
            // Initialize diagrams in this tab if not already rendered
            const hasRendered = targetPane.getAttribute('data-rendered');
            if (!hasRendered) {
                await this.initMermaidInTab(targetPane);
                targetPane.setAttribute('data-rendered', 'true');
            } else {
                // Already rendered, just adjust heights
                setTimeout(() => {
                    const containers = targetPane.querySelectorAll('.diagram-container');
                    containers.forEach(container => this.adjustDiagramHeight(container));
                    this.applyScale(this.currentScale);
                }, 50);
            }
            
            // Also handle any new diagrams that might be added
            const scaleDisplay = targetPane.querySelector('.scale-display');
            if (scaleDisplay) {
                scaleDisplay.textContent = '100%';
            }
        }
    }

    // Setup event listeners
    setupEventListeners() {
        // Tab switching
        const tabs = document.querySelectorAll('.nav-tab');
        tabs.forEach(tab => {
            tab.addEventListener('click', (e) => {
                const tabId = tab.getAttribute('data-tab');
                if (tabId) this.showTab(tabId);
            });
        });
        
        // Scale controls - delegated event handling
        document.addEventListener('click', (e) => {
            const target = e.target;
            const activePane = document.querySelector('.tab-pane.active');
            if (!activePane) return;
            
            // Find scale buttons within active pane or global
            const deltaBtn = target.closest('[data-scale-delta]');
            if (deltaBtn && (activePane.contains(deltaBtn) || !deltaBtn.closest('.tab-pane'))) {
                const delta = parseFloat(deltaBtn.getAttribute('data-scale-delta'));
                if (!isNaN(delta)) {
                    this.changeScale(delta);
                }
                return;
            }
            
            const resetBtn = target.closest('[data-reset-scale]');
            if (resetBtn && (activePane.contains(resetBtn) || !resetBtn.closest('.tab-pane'))) {
                this.resetScale();
                return;
            }
        });
        
        // Handle window resize - re-adjust heights
        let resizeTimeout;
        window.addEventListener('resize', () => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                const activePane = document.querySelector('.tab-pane.active');
                if (activePane) {
                    const containers = activePane.querySelectorAll('.diagram-container');
                    containers.forEach(container => this.adjustDiagramHeight(container));
                    this.applyScale(this.currentScale);
                }
            }, 200);
        });
    }

    // Initialize on load
    async init() {
        // Configure mermaid
        mermaid.initialize({
            startOnLoad: false,
            securityLevel: 'loose',
            theme: 'base',
            themeVariables: {
                primaryColor: '#667eea',
                primaryTextColor: '#333',
                primaryBorderColor: '#667eea',
                lineColor: '#667eea',
                secondaryColor: '#764ba2',
                tertiaryColor: '#f8f9fa',
                fontSize: '14px'
            },
            flowchart: {
                useMaxWidth: false,
                htmlLabels: true,
                curve: 'basis'
            },
            sequence: {
                useMaxWidth: false,
                diagramMarginX: 50,
                diagramMarginY: 10
            }
        });
        
        this.setupEventListeners();
        
        // Initial load - render first tab
        await this.showTab('overview');
        
        // Setup mutation observer for dynamic content
        this.setupMutationObserver();
    }

    // Setup mutation observer for dynamic content changes
    setupMutationObserver() {
        this.observer = new MutationObserver((mutations) => {
            const activePane = document.querySelector('.tab-pane.active');
            if (activePane) {
                mutations.forEach((mutation) => {
                    if (mutation.type === 'childList' || mutation.type === 'subtree') {
                        const newSvgs = activePane.querySelectorAll('.mermaid svg:not([data-height-adjusted])');
                        if (newSvgs.length > 0) {
                            newSvgs.forEach(svg => svg.setAttribute('data-height-adjusted', 'true'));
                            setTimeout(() => {
                                const containers = activePane.querySelectorAll('.diagram-container');
                                containers.forEach(container => this.adjustDiagramHeight(container));
                            }, 80);
                        }
                    }
                });
            }
        });
        
        this.observer.observe(document.body, { childList: true, subtree: true });
    }

    // Cleanup method
    destroy() {
        if (this.observer) {
            this.observer.disconnect();
        }
    }
}

// Auto-initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.mermaidHeightAdjuster = new MermaidHeightAdjuster();
});

// Export for manual usage if needed
window.MermaidHeightAdjuster = MermaidHeightAdjuster;
