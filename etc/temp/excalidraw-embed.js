// Excalidraw Embed Component
// Auto-initializes Excalidraw diagrams from data attributes

class ExcalidrawEmbed {
    
    constructor() {
        this.renderer = null;
        this.initialized = false;
    }
    
    // Initialize the embed system
    init(options = {}) {
        if (this.initialized) return;
        
        console.log('🚀 Initializing ExcalidrawEmbed');
        
        // Wait for dependencies
        this.waitForDependencies().then(() => {
            this.renderer = new window.ExcalidrawRenderer(options);
            this.scanAndRender();
            this.setupObserver();
            this.initialized = true;
            console.log('✅ ExcalidrawEmbed ready');
        }).catch(error => {
            console.error('❌ Failed to initialize ExcalidrawEmbed:', error);
        });
    }
    
    // Wait for all dependencies to load
    async waitForDependencies() {
        const maxWait = 10000; // 10 seconds
        const checkInterval = 100; // 100ms
        let waited = 0;
        
        while (waited < maxWait) {
            if (typeof window.ExcalidrawRenderer !== 'undefined' && 
                typeof window.ObsidianExcalidrawDecompressor !== 'undefined') {
                return;
            }
            await new Promise(resolve => setTimeout(resolve, checkInterval));
            waited += checkInterval;
        }
        
        throw new Error('Dependencies not loaded within timeout');
    }
    
    // Scan DOM and render all excalidraw-embed elements
    scanAndRender() {
        const elements = document.querySelectorAll('[data-excalidraw-src], [data-excalidraw-data]');
        console.log(`🔍 Found ${elements.length} Excalidraw embed elements`);
        
        elements.forEach(element => this.renderElement(element));
    }
    
    // Setup MutationObserver for dynamically added elements
    setupObserver() {
        const observer = new MutationObserver(mutations => {
            mutations.forEach(mutation => {
                mutation.addedNodes.forEach(node => {
                    if (node.nodeType === Node.ELEMENT_NODE) {
                        // Check if the node itself has excalidraw attributes
                        if (node.hasAttribute && (node.hasAttribute('data-excalidraw-src') || node.hasAttribute('data-excalidraw-data'))) {
                            this.renderElement(node);
                        }
                        // Check for child nodes with excalidraw attributes
                        const childElements = node.querySelectorAll && node.querySelectorAll('[data-excalidraw-src], [data-excalidraw-data]');
                        if (childElements) {
                            childElements.forEach(child => this.renderElement(child));
                        }
                    }
                });
            });
        });
        
        observer.observe(document.body, {
            childList: true,
            subtree: true
        });
    }
    
    // Render individual element
    async renderElement(element) {
        try {
            const src = element.getAttribute('data-excalidraw-src');
            const data = element.getAttribute('data-excalidraw-data');
            const containerId = element.id || this.generateId();
            
            // Ensure element has an ID
            if (!element.id) {
                element.id = containerId;
            }
            
            // Add loading state
            element.innerHTML = '<div style="padding: 20px; text-align: center; color: #666;">🔄 Loading Excalidraw diagram...</div>';
            
            if (src) {
                // Load from URL/file
                await this.renderer.renderFromUrl(src, containerId);
            } else if (data) {
                // Load from inline JSON data
                const excalidrawData = JSON.parse(data);
                this.renderer.render(excalidrawData, containerId);
            } else {
                throw new Error('No data-excalidraw-src or data-excalidraw-data attribute found');
            }
            
            // Add success class
            element.classList.add('excalidraw-rendered');
            
        } catch (error) {
            console.error('❌ Failed to render element:', error);
            element.innerHTML = `
                <div style="padding: 15px; border: 1px solid #ff6b6b; background: #ffe0e0; border-radius: 4px; font-size: 0.9em;">
                    <strong>❌ Failed to load Excalidraw diagram</strong><br>
                    ${error.message}
                </div>
            `;
            element.classList.add('excalidraw-error');
        }
    }
    
    // Generate unique ID
    generateId() {
        return 'excalidraw-' + Math.random().toString(36).substr(2, 9);
    }
    
    // Public method to manually render an element
    renderElementById(elementId) {
        const element = document.getElementById(elementId);
        if (element) {
            this.renderElement(element);
        } else {
            console.error('Element not found:', elementId);
        }
    }
    
    // Public method to render from file input
    async renderFromFileInput(fileInput, targetElementId) {
        const file = fileInput.files[0];
        if (file && this.renderer) {
            await this.renderer.renderFromFile(file, targetElementId);
        }
    }
}

// Auto-initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    window.excalidrawEmbed = new ExcalidrawEmbed();
    window.excalidrawEmbed.init();
});

// Export for manual use
window.ExcalidrawEmbed = ExcalidrawEmbed;
