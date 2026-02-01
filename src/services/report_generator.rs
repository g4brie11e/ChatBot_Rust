use super::session_manager::SessionData;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

pub async fn generate_pdf_report(session_id: &str, data: &SessionData) -> std::io::Result<String> {
    let dir = "public/reports";
    tokio::fs::create_dir_all(dir).await?;
    
    let file_path = format!("{}/{}.pdf", dir, session_id);
    let relative_path = format!("/reports/{}.pdf", session_id);
    
    // Clone data to move into the blocking thread
    let data = data.clone();
    let session_id = session_id.to_string();
    let file_path_clone = file_path.clone();

    // Run PDF generation in a blocking task (CPU intensive)
    tokio::task::spawn_blocking(move || {
        let (doc, page1, layer1) = PdfDocument::new("Project Report", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);
        
        // Use built-in fonts (no external file needed)
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

        let mut y = 270.0; // Start from top (A4 is 297mm high)
        
        // Title
        current_layer.use_text("Project Request Report", 24.0, Mm(20.0), Mm(y), &font_bold);
        y -= 20.0;

        // Fields
        let fields = vec![
            ("Client Name", data.name.as_deref().unwrap_or("Unknown")),
            ("Email Address", data.email.as_deref().unwrap_or("N/A")),
            ("Budget Estimate", data.budget.as_deref().unwrap_or("N/A")),
        ];

        for (label, value) in fields {
            current_layer.use_text(label, 12.0, Mm(20.0), Mm(y), &font_bold);
            current_layer.use_text(value, 12.0, Mm(70.0), Mm(y), &font);
            y -= 10.0;
        }

        // Topics
        y -= 5.0;
        current_layer.use_text("Detected Topics:", 12.0, Mm(20.0), Mm(y), &font_bold);
        y -= 10.0;
        
        let topics_str = if data.detected_keywords.is_empty() {
            "None".to_string()
        } else {
            data.detected_keywords.join(", ").to_uppercase()
        };
        current_layer.use_text(topics_str, 12.0, Mm(20.0), Mm(y), &font);

        // Footer
        current_layer.use_text(format!("Session ID: {}", session_id), 10.0, Mm(20.0), Mm(20.0), &font);

        let file = File::create(file_path_clone).unwrap();
        let mut writer = BufWriter::new(file);
        doc.save(&mut writer).unwrap();
    }).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(relative_path)
}