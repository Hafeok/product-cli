//! The IRIs extraction writes — RDF, RDFS, OWL, SKOS, PROV, plus the
//! registry's own track vocabulary.
//!
//! Full IRIs, never prefixed names: the writer binds prefixes for reading
//! comfort, but every term the mapping rows build is unambiguous here so a
//! prefix change can never silently retarget an assertion.

/// `rdf:type`.
pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
/// `rdfs:subClassOf`.
pub const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
/// `rdfs:domain`.
pub const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
/// `rdfs:range`.
pub const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
/// `rdfs:label`.
pub const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
/// `owl:Class`.
pub const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
/// `owl:DatatypeProperty`.
pub const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
/// `owl:ObjectProperty`.
pub const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
/// `skos:altLabel`.
pub const SKOS_ALT_LABEL: &str = "http://www.w3.org/2004/02/skos/core#altLabel";

/// The prefixes the writer binds, longest IRI first so matching is greedy.
pub const PREFIXES: [(&str, &str); 6] = [
    ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
    ("owl", "http://www.w3.org/2002/07/owl#"),
    ("skos", "http://www.w3.org/2004/02/skos/core#"),
    ("prov", "http://www.w3.org/ns/prov#"),
    ("xsd", "http://www.w3.org/2001/XMLSchema#"),
];

/// XSD datatype IRIs, built from the one namespace.
pub fn xsd(name: &str) -> String {
    format!("http://www.w3.org/2001/XMLSchema#{name}")
}

/// The registry's own terms, under the instance's base.
pub mod reg {
    /// `reg:dependsOn` — the usage-inferred relation the CONSTRUCT rules derive.
    pub fn depends_on(base: &str) -> String {
        format!("{base}dependsOn")
    }

    /// `reg:inModule` — structural containment in a module axis.
    pub fn in_module(base: &str) -> String {
        format!("{base}inModule")
    }

    /// `reg:Axis` — the class a module axis is an instance of.
    pub fn axis_class(base: &str) -> String {
        format!("{base}Axis")
    }

    /// The local name of the individual naming one LSP operation, as
    /// `reg:standsOn` points at it. Prefixed so an operation name can never
    /// collide with a grade or a source kind in the same namespace.
    pub fn operation_name(operation: &str) -> String {
        format!("op-{operation}")
    }
}

/// The C# primitive-to-XSD map, as far as it honestly goes.
///
/// Deliberately partial. A type absent here yields a property with **no**
/// range rather than a guessed one: an unranged property is a legible gap,
/// and a wrong range is a claim about the domain nobody made.
pub fn primitive_datatype(type_name: &str) -> Option<String> {
    let xsd_name = match type_name {
        "string" | "String" | "char" => "string",
        "int" | "Int32" => "int",
        "long" | "Int64" => "long",
        "short" | "Int16" => "short",
        "byte" | "Byte" => "unsignedByte",
        "bool" | "Boolean" => "boolean",
        "decimal" | "Decimal" => "decimal",
        "double" | "Double" => "double",
        "float" | "Single" => "float",
        "DateTime" | "DateTimeOffset" => "dateTime",
        "DateOnly" => "date",
        "TimeOnly" | "TimeSpan" => "time",
        _ => return None,
    };
    Some(xsd(xsd_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bound_prefix_is_distinct_in_name_and_target() {
        let mut names: Vec<&str> = PREFIXES.iter().map(|(n, _)| *n).collect();
        let mut iris: Vec<&str> = PREFIXES.iter().map(|(_, i)| *i).collect();
        names.sort_unstable();
        iris.sort_unstable();
        let (n, i) = (names.len(), iris.len());
        names.dedup();
        iris.dedup();
        assert_eq!((names.len(), iris.len()), (n, i));
    }

    #[test]
    fn an_unmapped_type_gets_no_range_rather_than_a_guess() {
        assert_eq!(primitive_datatype("string"), Some(xsd("string")));
        assert_eq!(primitive_datatype("int"), Some(xsd("int")));
        // A value-object identity, a GUID, an unloaded project's type: all
        // honestly unranged rather than coerced to xsd:string.
        assert_eq!(primitive_datatype("Guid"), None);
        assert_eq!(primitive_datatype("CustomerId"), None);
    }

    #[test]
    fn the_vocabulary_terms_sit_under_the_instance_base() {
        let base = "tag:x,2026:ground/";
        assert_eq!(reg::depends_on(base), "tag:x,2026:ground/dependsOn");
        assert_eq!(reg::axis_class(base), "tag:x,2026:ground/Axis");
    }
}
