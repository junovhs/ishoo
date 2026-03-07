import re

content = open('assets/style.css').read()

def repl(m):
    val = int(m.group(1))
    new_val = int(round(val * 1.2))
    return f"font-size: {new_val}px"

content = re.sub(r'font-size:\s*(\d+)px', repl, content)

# Remove the broken border
content = content.replace("border: 1px solid transparent;", "")

# Add antialiasing to body
content = content.replace("body {\n", "body {\n    -webkit-font-smoothing: antialiased;\n    -moz-osx-font-smoothing: grayscale;\n")

# Re-adjust card heights for the 20% taller text
content = content.replace("--card-h: 70px;", "--card-h: 85px;")
content = content.replace("--card-h: 28px;", "--card-h: 36px;")

open('assets/style.css', 'w').write(content)
