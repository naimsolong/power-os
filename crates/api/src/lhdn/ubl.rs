// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use bigdecimal::BigDecimal;
use serde_json::{json, Map, Value};

use crate::routes::invoices::{InvoiceDetailResponse, InvoiceLineResponse};

/// Supplier details come from the workspace LHDN settings.
pub struct SupplierInfo<'a> {
    pub name: &'a str,
    pub tin: &'a str,
}

/// Buyer details come from the invoice party.
pub struct BuyerInfo<'a> {
    pub name: &'a str,
    pub tin: Option<&'a str>,
}

/// Build a simplified UBL 2.1 JSON document for LHDN MyInvois.
///
/// This is intentionally minimal: it includes only the mandatory structural
/// fields plus supplier/buyer TINs, line items and totals. A production
/// integration should expand this to cover tax, charges and Malaysia-specific
/// extensions.
pub fn build_ubl_json(
    invoice: &InvoiceDetailResponse,
    supplier: SupplierInfo<'_>,
    buyer: BuyerInfo<'_>,
) -> Value {
    let currency = &invoice.currency;
    let zero = BigDecimal::from(0);

    let tax_total = json!({
        "TaxAmount": [{ "_": fmt_amount(&zero), "currencyID": currency }],
        "TaxSubtotal": [{
            "TaxableAmount": [{ "_": fmt_amount(&invoice.total_amount), "currencyID": currency }],
            "TaxAmount": [{ "_": fmt_amount(&zero), "currencyID": currency }],
            "TaxCategory": [{
                "ID": [{ "_": "E" }],
                "Percent": [{ "_": 0 }],
                "TaxScheme": [{ "ID": [{ "_": "GST" }] }]
            }]
        }]
    });

    let legal_monetary_total = json!({
        "LineExtensionAmount": [{ "_": fmt_amount(&invoice.total_amount), "currencyID": currency }],
        "TaxExclusiveAmount": [{ "_": fmt_amount(&invoice.total_amount), "currencyID": currency }],
        "TaxInclusiveAmount": [{ "_": fmt_amount(&invoice.total_amount), "currencyID": currency }],
        "PayableAmount": [{ "_": fmt_amount(&invoice.total_amount), "currencyID": currency }]
    });

    let invoice_lines: Vec<Value> = invoice
        .lines
        .iter()
        .enumerate()
        .map(|(idx, line)| build_invoice_line(idx + 1, line, currency))
        .collect();

    let mut root = Map::new();
    root.insert(
        "_D".to_string(),
        Value::String("urn:oasis:names:specification:ubl:schema:xsd:Invoice-2".to_string()),
    );
    root.insert("ID".to_string(), json!([{ "_": &invoice.invoice_number }]));
    root.insert(
        "IssueDate".to_string(),
        json!([{ "_": invoice.issue_date.to_string() }]),
    );
    if let Some(due) = invoice.due_date {
        root.insert("DueDate".to_string(), json!([{ "_": due.to_string() }]));
    }
    root.insert("InvoiceTypeCode".to_string(), json!([{ "_": "01" }]));
    root.insert(
        "DocumentCurrencyCode".to_string(),
        json!([{ "_": currency }]),
    );
    root.insert(
        "AccountingSupplierParty".to_string(),
        json!([{ "Party": [build_party(supplier.name, supplier.tin)] }]),
    );
    root.insert(
        "AccountingCustomerParty".to_string(),
        json!([{
            "Party": [build_party(
                buyer.name,
                buyer.tin.unwrap_or("NA")
            )]
        }]),
    );
    root.insert("TaxTotal".to_string(), json!([tax_total]));
    root.insert(
        "LegalMonetaryTotal".to_string(),
        json!([legal_monetary_total]),
    );
    root.insert("InvoiceLine".to_string(), Value::Array(invoice_lines));

    Value::Object(root)
}

fn build_party(name: &str, tin: &str) -> Value {
    json!({
        "PartyLegalEntity": [{
            "RegistrationName": [{ "_": name }]
        }],
        "PartyIdentification": [{
            "ID": [{ "_": tin, "schemeID": "TIN" }]
        }],
        "PartyName": [{ "Name": [{ "_": name }] }],
        "PostalAddress": [{
            "Country": [{ "IdentificationCode": [{ "_": "MYS" }] }]
        }]
    })
}

fn build_invoice_line(index: usize, line: &InvoiceLineResponse, currency: &str) -> Value {
    json!({
        "ID": [{ "_": index.to_string() }],
        "InvoicedQuantity": [{ "_": line.quantity.to_string(), "unitCode": "C62" }],
        "LineExtensionAmount": [{ "_": line.line_total.to_string(), "currencyID": currency }],
        "Item": [{
            "Description": [{ "_": &line.description }],
            "TaxTotal": [{
                "TaxAmount": [{ "_": "0", "currencyID": currency }]
            }]
        }],
        "Price": [{
            "PriceAmount": [{ "_": line.unit_price.to_string(), "currencyID": currency }]
        }]
    })
}

fn fmt_amount(value: &BigDecimal) -> String {
    value.to_string()
}
