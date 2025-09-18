// Excalidraw Renderer Module
// Main orchestrator for rendering Excalidraw diagrams from various sources

class ExcalidrawRenderer {
    
    constructor(options = {}) {
        this.options = {
            theme: options.theme || 'light',
            viewModeEnabled: options.viewModeEnabled !== false, // default true
            zenModeEnabled: options.zenModeEnabled || false,
            gridModeEnabled: options.gridModeEnabled || false,
            fallbackToSVG: options.fallbackToSVG !== false, // default true
            ...options
        };
        
        this.isReactAvailable = typeof window.React !== 'undefined' && typeof window.ReactDOM !== 'undefined';
        this.isExcalidrawAvailable = typeof window.ExcalidrawLib !== 'undefined';
        
        console.log('🎨 ExcalidrawRenderer initialized', {
            react: this.isReactAvailable,
            excalidraw: this.isExcalidrawAvailable,
            options: this.options
        });
    }
    
    // Main public method: render from file
    async renderFromFile(file, containerId) {
        try {
            console.log('📁 Loading file:', file.name);
            
            // Step 1: Read file
            const content = await this.readFile(file);
            
            // Step 2: Extract and parse data
            const excalidrawData = this.extractAndParse(content);
            
            // Step 3: Render
            return this.render(excalidrawData, containerId);
            
        } catch (error) {
            console.error('❌ Failed to render from file:', error);
            throw error;
        }
    }
    
    // Main public method: render from URL
    async renderFromUrl(url, containerId) {
        try {
            console.log('🌐 Loading from URL:', url);
            
            // Step 1: Fetch content
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`Failed to fetch ${url}: ${response.status}`);
            }
            const content = await response.text();
            
            // Step 2: Extract and parse data
            const excalidrawData = this.extractAndParse(content);
            
            // Step 3: Render
            return this.render(excalidrawData, containerId);
            
        } catch (error) {
            console.error('❌ Failed to render from URL:', error);
            throw error;
        }
    }
    
    // Main public method: render from data
    render(excalidrawData, containerId) {
        try {
            console.log('🎨 Rendering Excalidraw data to container:', containerId);
            
            const container = typeof containerId === 'string' 
                ? document.getElementById(containerId)
                : containerId;
                
            if (!container) {
                throw new Error(`Container not found: ${containerId}`);
            }
            
            // Clear container
            container.innerHTML = '';
            
            // Validate data
            if (!this.validateExcalidrawData(excalidrawData)) {
                throw new Error('Invalid Excalidraw data format');
            }
            
            // Try React-based rendering first
            if (this.isReactAvailable && this.isExcalidrawAvailable) {
                return this.renderWithReact(excalidrawData, container);
            }
            
            // Fallback to SVG if enabled
            if (this.options.fallbackToSVG) {
                return this.renderWithSVG(excalidrawData, container);
            }
            
            throw new Error('No rendering method available');
            
        } catch (error) {
            console.error('❌ Rendering failed:', error);
            this.renderError(containerId, error);
            throw error;
        }
    }
    
    // Private: Read file content
    readFile(file) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = (e) => resolve(e.target.result);
            reader.onerror = () => reject(new Error('Failed to read file'));
            reader.readAsText(file);
        });
    }
    
    // Private: Extract and parse data from content
    extractAndParse(content) {
        console.log('🔍 Extracting Excalidraw data from content');
        
        // Determine content type and delegate to appropriate parser
        if (content.includes('```compressed-json')) {
            return window.ObsidianExcalidrawDecompressor.parseMarkdownForExcalidraw(content);
        } else if (content.includes('```json')) {
            return this.parseRegularJSON(content);
        } else if (content.trim().startsWith('{') && content.includes('"type":"excalidraw"')) {
            return JSON.parse(content);
        } else {
            throw new Error('Unsupported content format');
        }
    }
    
    // Private: Parse regular JSON from markdown
    parseRegularJSON(content) {
        const jsonBlockRegex = /```json[\r\n]+([\s\S]*?)[\r\n]+```/g;
        const match = jsonBlockRegex.exec(content);
        
        if (match && match[1]) {
            try {
                const parsed = JSON.parse(match[1]);
                if (parsed && parsed.type === 'excalidraw') {
                    console.log('✅ Regular JSON parse successful!');
                    return parsed;
                }
            } catch (e) {
                console.log('Regular JSON parse failed:', e.message);
            }
        }
        
        throw new Error('No valid JSON Excalidraw data found');
    }
    
    // Private: Validate Excalidraw data structure
    validateExcalidrawData(data) {
        return data && 
               typeof data === 'object' && 
               data.type === 'excalidraw' && 
               Array.isArray(data.elements);
    }
    
    // Private: Render with React
    renderWithReact(excalidrawData, container) {
        console.log('⚛️ Rendering with React');
        
        const { createElement } = window.React;
        const { createRoot } = window.ReactDOM;
        
        const excalidrawComponent = createElement(window.ExcalidrawLib.Excalidraw, {
            initialData: excalidrawData,
            viewModeEnabled: this.options.viewModeEnabled,
            zenModeEnabled: this.options.zenModeEnabled,
            gridModeEnabled: this.options.gridModeEnabled,
            theme: this.options.theme
        });
        
        const root = createRoot(container);
        root.render(excalidrawComponent);
        
        console.log('✅ React rendering successful');
        return { method: 'react', container, data: excalidrawData };
    }
    
    // Private: Render with SVG fallback
    renderWithSVG(excalidrawData, container) {
        console.log('🎭 Rendering with SVG fallback');
        
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        svg.setAttribute("width", "100%");
        svg.setAttribute("height", "100%");
        svg.setAttribute("viewBox", "0 0 800 600");
        svg.style.border = "1px solid #ccc";
        
        // Create arrowhead marker
        const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
        const marker = document.createElementNS("http://www.w3.org/2000/svg", "marker");
        marker.setAttribute("id", "arrowhead");
        marker.setAttribute("markerWidth", "10");
        marker.setAttribute("markerHeight", "7");
        marker.setAttribute("refX", "9");
        marker.setAttribute("refY", "3.5");
        marker.setAttribute("orient", "auto");
        
        const polygon = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
        polygon.setAttribute("points", "0 0, 10 3.5, 0 7");
        polygon.setAttribute("fill", "black");
        
        marker.appendChild(polygon);
        defs.appendChild(marker);
        svg.appendChild(defs);
        
        // Render elements
        if (excalidrawData.elements) {
            excalidrawData.elements.forEach(element => {
                if (element.isDeleted) return;
                this.renderSVGElement(svg, element);
            });
        }
        
        container.appendChild(svg);
        console.log('✅ SVG rendering successful');
        return { method: 'svg', container, data: excalidrawData };
    }
    
    // Private: Render individual SVG element
    renderSVGElement(svg, element) {
        switch (element.type) {
            case "rectangle":
                const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
                rect.setAttribute("x", element.x);
                rect.setAttribute("y", element.y);
                rect.setAttribute("width", element.width);
                rect.setAttribute("height", element.height);
                rect.setAttribute("fill", element.backgroundColor || "white");
                rect.setAttribute("stroke", element.strokeColor || "black");
                rect.setAttribute("stroke-width", element.strokeWidth || 2);
                
                if (element.text) {
                    const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
                    text.setAttribute("x", element.x + element.width / 2);
                    text.setAttribute("y", element.y + element.height / 2);
                    text.setAttribute("text-anchor", "middle");
                    text.setAttribute("dominant-baseline", "middle");
                    text.setAttribute("font-family", "Arial");
                    text.setAttribute("font-size", element.fontSize || 16);
                    text.textContent = element.text;
                    svg.appendChild(text);
                }
                
                svg.appendChild(rect);
                break;
                
            case "ellipse":
                const ellipse = document.createElementNS("http://www.w3.org/2000/svg", "ellipse");
                ellipse.setAttribute("cx", element.x + element.width / 2);
                ellipse.setAttribute("cy", element.y + element.height / 2);
                ellipse.setAttribute("rx", element.width / 2);
                ellipse.setAttribute("ry", element.height / 2);
                ellipse.setAttribute("fill", element.backgroundColor || "white");
                ellipse.setAttribute("stroke", element.strokeColor || "black");
                ellipse.setAttribute("stroke-width", element.strokeWidth || 2);
                
                if (element.text) {
                    const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
                    text.setAttribute("x", element.x + element.width / 2);
                    text.setAttribute("y", element.y + element.height / 2);
                    text.setAttribute("text-anchor", "middle");
                    text.setAttribute("dominant-baseline", "middle");
                    text.setAttribute("font-family", "Arial");
                    text.setAttribute("font-size", element.fontSize || 16);
                    text.textContent = element.text;
                    svg.appendChild(text);
                }
                
                svg.appendChild(ellipse);
                break;
                
            case "arrow":
                if (element.points && element.points.length >= 2) {
                    const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
                    line.setAttribute("x1", element.x + element.points[0][0]);
                    line.setAttribute("y1", element.y + element.points[0][1]);
                    line.setAttribute("x2", element.x + element.points[element.points.length - 1][0]);
                    line.setAttribute("y2", element.y + element.points[element.points.length - 1][1]);
                    line.setAttribute("stroke", element.strokeColor || "black");
                    line.setAttribute("stroke-width", element.strokeWidth || 2);
                    if (element.endArrowhead) {
                        line.setAttribute("marker-end", "url(#arrowhead)");
                    }
                    svg.appendChild(line);
                }
                break;
                
            case "text":
                const textElement = document.createElementNS("http://www.w3.org/2000/svg", "text");
                textElement.setAttribute("x", element.x);
                textElement.setAttribute("y", element.y);
                textElement.setAttribute("font-family", "Arial");
                textElement.setAttribute("font-size", element.fontSize || 16);
                textElement.setAttribute("fill", element.strokeColor || "black");
                textElement.textContent = element.text || element.originalText || "";
                svg.appendChild(textElement);
                break;
        }
    }
    
    // Private: Render error message
    renderError(containerId, error) {
        const container = typeof containerId === 'string' 
            ? document.getElementById(containerId)
            : containerId;
            
        if (container) {
            container.innerHTML = `
                <div style="padding: 20px; border: 1px solid #ff6b6b; background: #ffe0e0; border-radius: 4px;">
                    <h3 style="margin: 0 0 10px 0; color: #d63031;">❌ Rendering Error</h3>
                    <p style="margin: 0; color: #2d3436;"><strong>Error:</strong> ${error.message}</p>
                    <p style="margin: 10px 0 0 0; color: #636e72; font-size: 0.9em;">
                        Please ensure the file contains valid Excalidraw data.
                    </p>
                </div>
            `;
        }
    }
}

// Export for global use
window.ExcalidrawRenderer = ExcalidrawRenderer;
