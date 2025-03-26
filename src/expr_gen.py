def main():
    f = open("expr.rs", "x")
    code = define_ast([("Binary", ["left: Box<dyn Expr>", "operator: Token", "right Box<dyn Expr>"]),
                       ("Grouping", ["expression: Box<dyn Expr>"]),
                ("Literal", ["value: Box<dyn Literal>"]),
                ("Unary", ["operator: Token", "right: Box<Expr>"])
                ])
    f.write(code)


def define_ast(productions: list[(str, list[str])]) -> str:
    rust_code = ""
    rust_code += "trait Expr {} \n\n"

    for prod in productions:
        (class_name, fields) = prod
        rust_code += f"struct {class_name} {'{'}\n"
        for field in fields:
            rust_code += f"\t {field},\n"
        rust_code += "}\n"
        rust_code += f"impl Expr for {class_name} {{}}"
        rust_code += f"\n\n"

    return rust_code

if __name__ == '__main__':
    main()
