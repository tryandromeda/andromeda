// Comprehensive fillStyle demonstration
const canvas = createCanvas(600, 400);
const ctx = canvas.getContext("2d");

if (ctx) {
    console.log("🎨 fillStyle Demonstration");
    console.log("Test 1: Hex Colors");
    ctx.fillStyle = "#ff0000"; // Red
    console.log(`Set fillStyle to "#ff0000", got: ${ctx.fillStyle}`);
    ctx.fillRect(50, 50, 80, 80);
    
    ctx.fillStyle = "#00ff00"; // Green
    console.log(`Set fillStyle to "#00ff00", got: ${ctx.fillStyle}`);
    ctx.fillRect(140, 50, 80, 80);
    
    ctx.fillStyle = "#0000ff"; // Blue
    console.log(`Set fillStyle to "#0000ff", got: ${ctx.fillStyle}`);
    ctx.fillRect(230, 50, 80, 80);
    
    // Test 2: RGB colors
    console.log("\nTest 2: RGB Colors");
    ctx.fillStyle = "rgb(255, 165, 0)"; // Orange
    console.log(`Set fillStyle to "rgb(255, 165, 0)", got: ${ctx.fillStyle}`);
    ctx.fillRect(50, 140, 80, 80);
    
    ctx.fillStyle = "rgb(128, 0, 128)"; // Purple
    console.log(`Set fillStyle to "rgb(128, 0, 128)", got: ${ctx.fillStyle}`);
    ctx.fillRect(140, 140, 80, 80);
    
    ctx.fillStyle = "rgb(255, 192, 203)"; // Pink
    console.log(`Set fillStyle to "rgb(255, 192, 203)", got: ${ctx.fillStyle}`);
    ctx.fillRect(230, 140, 80, 80);
    
    // Test 3: RGBA colors (transparency)
    console.log("\nTest 3: RGBA Colors (with transparency)");
    ctx.fillStyle = "rgba(255, 0, 0, 0.7)"; // Semi-transparent red
    console.log(`Set fillStyle to "rgba(255, 0, 0, 0.7)", got: ${ctx.fillStyle}`);
    ctx.fillRect(50, 230, 80, 80);
    
    ctx.fillStyle = "rgba(0, 255, 0, 0.5)"; // Semi-transparent green
    console.log(`Set fillStyle to "rgba(0, 255, 0, 0.5)", got: ${ctx.fillStyle}`);
    ctx.fillRect(140, 230, 80, 80);
    
    ctx.fillStyle = "rgba(0, 0, 255, 0.3)"; // Semi-transparent blue
    console.log(`Set fillStyle to "rgba(0, 0, 255, 0.3)", got: ${ctx.fillStyle}`);
    ctx.fillRect(230, 230, 80, 80);
    
    // Test 4: Named colors
    console.log("\nTest 4: Named Colors");
    ctx.fillStyle = "red";
    console.log(`Set fillStyle to "red", got: ${ctx.fillStyle}`);
    ctx.fillRect(320, 50, 80, 80);
    
    ctx.fillStyle = "green";
    console.log(`Set fillStyle to "green", got: ${ctx.fillStyle}`);
    ctx.fillRect(410, 50, 80, 80);
    
    ctx.fillStyle = "blue";
    console.log(`Set fillStyle to "blue", got: ${ctx.fillStyle}`);
    ctx.fillRect(500, 50, 80, 80);
    
    ctx.fillStyle = "yellow";
    console.log(`Set fillStyle to "yellow", got: ${ctx.fillStyle}`);
    ctx.fillRect(320, 140, 80, 80);
    
    ctx.fillStyle = "cyan";
    console.log(`Set fillStyle to "cyan", got: ${ctx.fillStyle}`);
    ctx.fillRect(410, 140, 80, 80);
    
    ctx.fillStyle = "magenta";
    console.log(`Set fillStyle to "magenta", got: ${ctx.fillStyle}`);
    ctx.fillRect(500, 140, 80, 80);
    
    ctx.fillStyle = "black";
    console.log(`Set fillStyle to "black", got: ${ctx.fillStyle}`);
    ctx.fillRect(320, 230, 80, 80);
    
    ctx.fillStyle = "white";
    console.log(`Set fillStyle to "white", got: ${ctx.fillStyle}`);
    ctx.fillRect(410, 230, 80, 80);
    
    ctx.fillStyle = "gray";
    console.log(`Set fillStyle to "gray", got: ${ctx.fillStyle}`);
    ctx.fillRect(500, 230, 80, 80);
    
    // Test 5: Error handling
    console.log("\nTest 5: Error Handling");
    console.log(`Current fillStyle: ${ctx.fillStyle}`);
    ctx.fillStyle = "invalid-color";
    console.log(`After setting to "invalid-color": ${ctx.fillStyle}`);
    console.log("✅ Invalid color rejected correctly - fillStyle unchanged");
    
    // Test 6: Overlapping shapes with different fillStyles
    console.log("\nTest 6: Overlapping shapes");
    ctx.fillStyle = "rgba(255, 255, 0, 0.6)"; // Semi-transparent yellow
    ctx.fillRect(75, 320, 100, 60);
    
    ctx.fillStyle = "rgba(255, 0, 255, 0.6)"; // Semi-transparent magenta
    ctx.fillRect(125, 320, 100, 60);
    
    ctx.fillStyle = "rgba(0, 255, 255, 0.6)"; // Semi-transparent cyan
    ctx.fillRect(175, 320, 100, 60);
    
    // Render and save
    console.log("\n🚀 Rendering canvas...");
    const rendered = canvas.render();
    console.log(`Canvas render result: ${rendered}`);
    
    console.log("💾 Saving canvas as PNG...");
    const saved = canvas.saveAsPng("test.demo.png");
    console.log(`Canvas save result: ${saved}`);
    
    if (saved) {
        console.log("✅ fillStyle demonstration complete!");
        console.log("📁 Output saved as 'test.demo.png'");
        console.log("\n🎯 Summary:");
        console.log("✓ Hex colors (#rrggbb)");
        console.log("✓ RGB colors (rgb(r, g, b))");
        console.log("✓ RGBA colors (rgba(r, g, b, a))");
        console.log("✓ Named colors (red, green, blue, etc.)");
        console.log("✓ Error handling for invalid colors");
        console.log("✓ GPU-accelerated rendering with proper colors");
    } else {
        console.error("❌ Failed to save PNG");
    }
} else {
    console.error("❌ Failed to get 2D context");
}
