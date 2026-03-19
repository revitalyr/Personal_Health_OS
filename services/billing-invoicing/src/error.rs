use thiserror::Error;

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("Invoice not found")]
    InvoiceNotFound,
    
    #[error("Service charge not found")]
    ServiceChargeNotFound,
    
    #[error("Payment not found")]
    PaymentNotFound,
    
    #[error("Insurance claim not found")]
    InsuranceClaimNotFound,
    
    #[error("Invoice number already exists")]
    InvoiceNumberExists,
    
    #[error("Cannot modify paid invoice")]
    CannotModifyPaidInvoice,
    
    #[error("Cannot delete invoice with payments")]
    CannotDeleteInvoiceWithPayments,
    
    #[error("Payment amount exceeds invoice total")]
    PaymentExceedsTotal,
    
    #[error("Invalid invoice data: {0}")]
    InvalidData(String),
    
    #[error("Tax calculation error")]
    TaxCalculationError,
    
    #[error("PDF generation error")]
    PDFGenerationError,
    
    #[error("Email sending failed")]
    EmailSendingFailed,
    
    #[error("Payment processing failed")]
    PaymentProcessingFailed,
    
    #[error("Insurance claim submission failed")]
    ClaimSubmissionFailed,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Service temporarily unavailable")]
    ServiceUnavailable,
}
