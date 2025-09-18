// LZ-String Decompressor for Obsidian Excalidraw Files
// This handles the specific compression format used by Obsidian Excalidraw plugin

class ObsidianExcalidrawDecompressor {
    
    // Main decompression function for Obsidian Excalidraw compressed-json
    static decompressExcalidrawData(compressedString) {
        try {
            console.log("🔄 Attempting LZ-String decompression...");
            
            // Clean the compressed data (remove whitespace)
            const cleanedData = compressedString.replace(/\s+/g, '');
            console.log("Cleaned data length:", cleanedData.length);
            
            // Check if LZString is available
            if (typeof LZString === 'undefined') {
                throw new Error('LZ-String library not loaded');
            }
            
            // Try LZ-String decompression methods in order
            const methods = [
                { name: 'decompressFromBase64', func: LZString.decompressFromBase64 },
                { name: 'decompressFromEncodedURIComponent', func: LZString.decompressFromEncodedURIComponent },
                { name: 'decompressFromUTF16', func: LZString.decompressFromUTF16 },
                { name: 'decompress', func: LZString.decompress }
            ];
            
            for (const method of methods) {
                try {
                    console.log(`Trying ${method.name}...`);
                    const decompressed = method.func(cleanedData);
                    
                    if (decompressed && decompressed.length > 0) {
                        console.log(`${method.name} returned data, length:`, decompressed.length);
                        
                        // Try to parse as JSON
                        try {
                            const parsed = JSON.parse(decompressed);
                            if (parsed && parsed.type === 'excalidraw') {
                                console.log(`✅ ${method.name} decompression successful!`);
                                return parsed;
                            } else {
                                console.log(`${method.name} decompressed but not valid Excalidraw data`);
                            }
                        } catch (jsonError) {
                            console.log(`${method.name} decompressed but JSON parse failed:`, jsonError.message);
                        }
                    } else {
                        console.log(`${method.name} returned null or empty`);
                    }
                } catch (methodError) {
                    console.log(`${method.name} failed:`, methodError.message);
                }
            }
            
            throw new Error('All LZ-String decompression methods failed');
            
        } catch (error) {
            console.error('❌ LZ-String decompression failed:', error);
            throw error;
        }
    }
    
    // Parse markdown content to extract compressed-json blocks
    static parseMarkdownForExcalidraw(markdownContent) {
        try {
            console.log("🔍 Parsing markdown for Excalidraw data...");
            
            // Look for compressed-json code blocks
            const compressedJsonRegex = /```compressed-json[\r\n]+([\s\S]*?)[\r\n]+```/g;
            const match = compressedJsonRegex.exec(markdownContent);
            
            if (match && match[1]) {
                console.log("📦 Found compressed-json block");
                return this.decompressExcalidrawData(match[1]);
            }
            
            // Fallback: Look for regular JSON blocks
            const jsonBlockRegex = /```json[\r\n]+([\s\S]*?)[\r\n]+```/g;
            const jsonMatch = jsonBlockRegex.exec(markdownContent);
            
            if (jsonMatch && jsonMatch[1]) {
                console.log("📄 Found regular JSON block");
                try {
                    const parsed = JSON.parse(jsonMatch[1]);
                    if (parsed && parsed.type === 'excalidraw') {
                        console.log("✅ Regular JSON parse successful!");
                        return parsed;
                    }
                } catch (e) {
                    console.log("Regular JSON parse failed:", e.message);
                }
            }
            
            throw new Error('No valid Excalidraw data found in markdown');
            
        } catch (error) {
            console.error('❌ Markdown parsing failed:', error);
            throw error;
        }
    }
    
    // Validate Excalidraw data structure
    static validateExcalidrawData(data) {
        return data && 
               typeof data === 'object' && 
               data.type === 'excalidraw' && 
               Array.isArray(data.elements);
    }
}

// Export for global use
window.ObsidianExcalidrawDecompressor = ObsidianExcalidrawDecompressor;
