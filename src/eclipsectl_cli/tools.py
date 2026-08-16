"""MCP tool definitions for Eclipse skill (Claude Code integration)."""

from typing import Any, Dict, List

# Tool definitions following MCP protocol
# Based on SKILL_DESIGN.md - excludes file operations (use Claude Code's Write/Edit/Read instead)

TOOLS: List[Dict[str, Any]] = [
    # ===== Workspace =====
    {
        "name": "eclipse_refresh_file",
        "description": "Refresh a file in Eclipse workspace after modifications. Use this after Write/Edit operations to sync Eclipse.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Path to the file to refresh (WSL or Windows format)"
                }
            },
            "required": ["file"]
        }
    },
    {
        "name": "eclipse_refresh_workspace",
        "description": "Refresh entire Eclipse workspace. Use when multiple files changed or to ensure full sync.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },

    # ===== Project Management =====
    {
        "name": "eclipse_list_projects",
        "description": "List all projects in Eclipse workspace.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_get_project_layout",
        "description": "Get the directory structure and layout of a project.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                }
            },
            "required": ["project"]
        }
    },
    {
        "name": "eclipse_get_project_properties",
        "description": "Get project properties and configuration (build path, classpath, nature, etc.).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                }
            },
            "required": ["project"]
        }
    },

    # ===== Code Analysis =====
    {
        "name": "eclipse_get_javadoc",
        "description": "Get JavaDoc documentation for a fully qualified class name.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fqn": {
                    "type": "string",
                    "description": "Fully qualified class name (e.g., 'com.example.MyClass')"
                }
            },
            "required": ["fqn"]
        }
    },
    {
        "name": "eclipse_get_source",
        "description": "Get source code for a fully qualified class name.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fqn": {
                    "type": "string",
                    "description": "Fully qualified class name (e.g., 'com.example.MyClass')"
                }
            },
            "required": ["fqn"]
        }
    },
    {
        "name": "eclipse_get_call_hierarchy",
        "description": "Get method call hierarchy (callers and callees) for a method.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fqn": {
                    "type": "string",
                    "description": "Fully qualified class name (e.g., 'com.example.MyClass')"
                },
                "method": {
                    "type": "string",
                    "description": "Method name to analyze"
                }
            },
            "required": ["fqn", "method"]
        }
    },

    # ===== Build & Errors =====
    {
        "name": "eclipse_maven_build",
        "description": "Run Maven build with specified goals (e.g., 'clean install', 'package', 'test').",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                },
                "goals": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Maven goals to execute (e.g., ['clean', 'install'])"
                }
            },
            "required": ["project", "goals"]
        }
    },
    {
        "name": "eclipse_get_compilation_errors",
        "description": "Get compilation errors and warnings for a project or all projects.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name (optional - if omitted, returns errors for all projects)"
                }
            },
            "required": []
        }
    },

    # ===== Tests =====
    {
        "name": "eclipse_run_all_tests",
        "description": "Run all JUnit tests in the workspace.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_run_package_tests",
        "description": "Run all tests in a specific package.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                },
                "package": {
                    "type": "string",
                    "description": "Package name (e.g., 'com.example.tests')"
                }
            },
            "required": ["project", "package"]
        }
    },
    {
        "name": "eclipse_run_class_tests",
        "description": "Run all tests in a specific test class.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                },
                "class": {
                    "type": "string",
                    "description": "Fully qualified class name (e.g., 'com.example.UserTest')"
                }
            },
            "required": ["project", "class"]
        }
    },
    {
        "name": "eclipse_run_test_method",
        "description": "Run a specific test method in a test class.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                },
                "class": {
                    "type": "string",
                    "description": "Fully qualified class name (e.g., 'com.example.UserTest')"
                },
                "method": {
                    "type": "string",
                    "description": "Test method name (e.g., 'testCreateUser')"
                }
            },
            "required": ["project", "class", "method"]
        }
    },

    # ===== Editor Operations =====
    {
        "name": "eclipse_get_current_file",
        "description": "Get the file currently opened in Eclipse editor.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_get_editor_selection",
        "description": "Get the currently selected text in Eclipse editor.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_format_code",
        "description": "Format Java code according to Eclipse formatter settings.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "Path to the Java file to format (WSL or Windows format)"
                }
            },
            "required": ["file"]
        }
    },

    # ===== Console =====
    {
        "name": "eclipse_get_console_output",
        "description": "Get Eclipse console output (build logs, test results, etc.).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "consoleName": {
                    "type": "string",
                    "description": "Optional specific console name (e.g., 'Maven Console')"
                }
            },
            "required": []
        }
    },

    # ===== Search & Navigation =====
    {
        "name": "eclipse_file_search",
        "description": "Search for text in workspace files.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "containingText": {
                    "type": "string",
                    "description": "Text to search for"
                },
                "fileNamePatterns": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional file name patterns to filter (e.g., ['*.java', '*.xml'])"
                }
            },
            "required": ["containingText"]
        }
    },
    {
        "name": "eclipse_file_search_regexp",
        "description": "Search workspace files using a regular expression.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern"
                },
                "fileNamePatterns": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional file name patterns to filter"
                }
            },
            "required": ["pattern"]
        }
    },
    {
        "name": "eclipse_find_files",
        "description": "Find workspace files matching glob patterns.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fileNamePatterns": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "File name patterns (e.g., ['*.java', 'pom.xml'])"
                },
                "maxResults": {
                    "type": "integer",
                    "description": "Maximum results (default: 200)",
                    "default": 200
                }
            },
            "required": []
        }
    },
    {
        "name": "eclipse_open_type",
        "description": "Search for a Java type by name using Eclipse's Open Type search.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Type name or pattern (e.g., 'HashMap', '*Service')"
                }
            },
            "required": ["name"]
        }
    },
    {
        "name": "eclipse_find_references",
        "description": "Find all references to a Java element (class, method, field).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "elementType": {
                    "type": "string",
                    "description": "Element type: 'type', 'method', or 'field'"
                },
                "elementName": {
                    "type": "string",
                    "description": "Fully qualified element name (e.g., 'com.example.MyClass.myMethod')"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project scope"
                },
                "scope": {
                    "type": "string",
                    "description": "Search scope: 'workspace' (default) or 'project'",
                    "default": "workspace"
                }
            },
            "required": ["elementType", "elementName"]
        }
    },
    {
        "name": "eclipse_get_type_hierarchy",
        "description": "Get type hierarchy (superclasses, subclasses, interfaces) for a Java class.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fullyQualifiedClassName": {
                    "type": "string",
                    "description": "Fully qualified class name (e.g., 'com.example.MyClass')"
                }
            },
            "required": ["fullyQualifiedClassName"]
        }
    },
    {
        "name": "eclipse_call_hierarchy",
        "description": "Get call hierarchy for a method (callers and callees).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fullyQualifiedClassName": {
                    "type": "string",
                    "description": "Fully qualified class name"
                },
                "methodName": {
                    "type": "string",
                    "description": "Method name"
                },
                "methodSignature": {
                    "type": "string",
                    "description": "Optional method signature for overloaded methods"
                }
            },
            "required": ["fullyQualifiedClassName", "methodName"]
        }
    },
    {
        "name": "eclipse_get_implementations",
        "description": "Find implementations of an interface or abstract class/method.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "fullyQualifiedClassName": {
                    "type": "string",
                    "description": "Fully qualified class name"
                },
                "methodName": {
                    "type": "string",
                    "description": "Optional method name to find implementations of"
                }
            },
            "required": ["fullyQualifiedClassName"]
        }
    },

    # ===== Problems & Quick Fixes =====
    {
        "name": "eclipse_get_problems",
        "description": "Get problems from Eclipse Problems panel (compilation errors, warnings).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "severity": {
                    "type": "string",
                    "description": "Filter: 'error' (default), 'warning', or 'all'",
                    "default": "error"
                },
                "maxResults": {
                    "type": "integer",
                    "description": "Maximum results (default: 10)",
                    "default": 10
                }
            },
            "required": []
        }
    },
    {
        "name": "eclipse_get_quick_fixes",
        "description": "Get available Quick Fixes for problems in a file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "lineNumber": {
                    "type": "integer",
                    "description": "Optional line number to filter"
                }
            },
            "required": ["filePath"]
        }
    },
    {
        "name": "eclipse_apply_quick_fix",
        "description": "Apply a Quick Fix to resolve a problem at a specific line.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "lineNumber": {
                    "type": "integer",
                    "description": "Line number of the problem"
                },
                "fixIndex": {
                    "type": "integer",
                    "description": "Index of the fix to apply (from get_quick_fixes)"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath", "lineNumber", "fixIndex"]
        }
    },
    {
        "name": "eclipse_get_markers",
        "description": "Get all markers (errors, warnings, infos) for a file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath"]
        }
    },

    # ===== Maven =====
    {
        "name": "eclipse_list_maven_projects",
        "description": "List all Maven projects in workspace.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_get_effective_pom",
        "description": "Get effective POM for a Maven project.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "projectName": {
                    "type": "string",
                    "description": "Maven project name"
                }
            },
            "required": ["projectName"]
        }
    },
    {
        "name": "eclipse_get_project_dependencies",
        "description": "Get Maven project dependencies.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "projectName": {
                    "type": "string",
                    "description": "Maven project name"
                }
            },
            "required": ["projectName"]
        }
    },
    {
        "name": "eclipse_find_test_classes",
        "description": "Find all test classes in a project.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "projectName": {
                    "type": "string",
                    "description": "Project name"
                }
            },
            "required": ["projectName"]
        }
    },

    # ===== Refactoring =====
    {
        "name": "eclipse_rename_file",
        "description": "Rename a file in the project.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file to rename"
                },
                "newFileName": {
                    "type": "string",
                    "description": "New file name"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath", "newFileName"]
        }
    },
    {
        "name": "eclipse_refactor_rename_java_type",
        "description": "Rename a Java class/interface/enum with all references updated.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "newTypeName": {
                    "type": "string",
                    "description": "New type name"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath", "newTypeName"]
        }
    },
    {
        "name": "eclipse_refactor_move_java_type",
        "description": "Move a Java class/interface/enum to a different package.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "targetPackage": {
                    "type": "string",
                    "description": "Target package name"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath", "targetPackage"]
        }
    },
    {
        "name": "eclipse_refactor_rename_package",
        "description": "Rename a Java package with all references updated.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "packageName": {
                    "type": "string",
                    "description": "Current package name"
                },
                "newPackageName": {
                    "type": "string",
                    "description": "New package name"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["packageName", "newPackageName"]
        }
    },
    {
        "name": "eclipse_move_resource",
        "description": "Move a file or folder to a different location.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "sourcePath": {
                    "type": "string",
                    "description": "Source path"
                },
                "targetPath": {
                    "type": "string",
                    "description": "Target path"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["sourcePath", "targetPath"]
        }
    },
    {
        "name": "eclipse_refactor_extract_method",
        "description": "Extract code lines into a new method.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "startLine": {
                    "type": "integer",
                    "description": "Start line number"
                },
                "endLine": {
                    "type": "integer",
                    "description": "End line number"
                },
                "methodName": {
                    "type": "string",
                    "description": "Name for the extracted method"
                },
                "visibility": {
                    "type": "string",
                    "description": "Visibility: 'private' (default), 'protected', 'public'",
                    "default": "private"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath", "startLine", "endLine", "methodName"]
        }
    },

    # ===== Code Generation =====
    {
        "name": "eclipse_generate_getters_setters",
        "description": "Generate getter and/or setter methods for fields in a Java class.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "className": {
                    "type": "string",
                    "description": "Optional class name (if multiple classes in file)"
                },
                "fieldNames": {
                    "type": "string",
                    "description": "Comma-separated field names (default: all fields)"
                },
                "generateGetters": {
                    "type": "boolean",
                    "description": "Generate getters (default: true)",
                    "default": True
                },
                "generateSetters": {
                    "type": "boolean",
                    "description": "Generate setters (default: true)",
                    "default": True
                }
            },
            "required": ["filePath"]
        }
    },
    {
        "name": "eclipse_generate_tostring_equals_hashcode",
        "description": "Generate toString(), equals(), and/or hashCode() methods.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "className": {
                    "type": "string",
                    "description": "Optional class name"
                },
                "fieldNames": {
                    "type": "string",
                    "description": "Comma-separated field names (default: all fields)"
                }
            },
            "required": ["filePath"]
        }
    },
    {
        "name": "eclipse_implement_override_methods",
        "description": "Implement or override methods from superclass/interfaces.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "className": {
                    "type": "string",
                    "description": "Optional class name"
                },
                "methodNames": {
                    "type": "string",
                    "description": "Comma-separated method names (default: all unimplemented)"
                }
            },
            "required": ["filePath"]
        }
    },

    # ===== Java Member Editing =====
    {
        "name": "eclipse_read_java",
        "description": "Lists the structure of a Java class: fields, methods, constructors, inner types with their line ranges and full source code. MUST be called before writeJava or removeJava.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file (e.g., 'src/com/example/MyClass.java')"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "className": {
                    "type": "string",
                    "description": "Target class name (dotted for inner classes, e.g., 'Outer.Inner'). Lists all top-level types if omitted."
                }
            },
            "required": ["filePath"]
        }
    },
    {
        "name": "eclipse_write_java",
        "description": "Writes a code member (method, field, constructor, inner class) into a Java class. Replaces existing member with same signature or appends if not found. Requires readJava first.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "newSource": {
                    "type": "string",
                    "description": "Complete source code of the member to write"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "className": {
                    "type": "string",
                    "description": "Target class name (dotted for inner classes). Defaults to primary type if omitted."
                }
            },
            "required": ["filePath", "newSource"]
        }
    },
    {
        "name": "eclipse_remove_java",
        "description": "Removes named members (fields, methods, inner types) from a Java class by name. Requires readJava first.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "members": {
                    "type": "string",
                    "description": "Comma-separated member names to remove (e.g., 'getData,setData,myField')"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "className": {
                    "type": "string",
                    "description": "Target class name (dotted for inner classes). Defaults to primary type if omitted."
                }
            },
            "required": ["filePath", "members"]
        }
    },
    {
        "name": "eclipse_remove_lines",
        "description": "Removes line ranges from a Java file. Requires readJava first.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "lines": {
                    "type": "string",
                    "description": "Line ranges to remove (e.g., '1-5,65,100-105'). Lines are 1-based."
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath", "lines"]
        }
    },
    {
        "name": "eclipse_reset_read_gate",
        "description": "Resets the readJava gate. If a filePath is provided, resets only that file. If omitted, resets all files.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to a specific file to reset, or omit to reset all files."
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": []
        }
    },

    # ===== Source Operations =====
    {
        "name": "eclipse_organize_imports",
        "description": "Organize imports in a Java file (remove unused, add missing).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["filePath"]
        }
    },
    {
        "name": "eclipse_organize_imports_in_package",
        "description": "Organize imports in all Java files of a package.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "packageName": {
                    "type": "string",
                    "description": "Package name (e.g., 'com.example.service')"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                }
            },
            "required": ["packageName"]
        }
    },
    {
        "name": "eclipse_format_file",
        "description": "Format a Java file using Eclipse formatter settings.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the Java file"
                },
                "projectName": {
                    "type": "string",
                    "description": "Optional project name"
                },
                "lines": {
                    "type": "string",
                    "description": "Line ranges to format (e.g., '10-20,105,150-190'). If omitted, formats entire file."
                }
            },
            "required": ["filePath"]
        }
    },
    {
        "name": "eclipse_create_directories",
        "description": "Create a directory (and any missing parent directories) within a project.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "projectName": {
                    "type": "string",
                    "description": "The name of the project where directories will be created"
                },
                "directoryPath": {
                    "type": "string",
                    "description": "The directory path relative to the project root (e.g., 'src/com/example/newpackage')"
                }
            },
            "required": ["projectName", "directoryPath"]
        }
    },
    {
        "name": "eclipse_search_and_replace",
        "description": "Search for a plain text string across workspace files and replace all occurrences with the replacement text.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "containingText": {
                    "type": "string",
                    "description": "The text to search for (plain substring, not regex)"
                },
                "replacementText": {
                    "type": "string",
                    "description": "The replacement text"
                },
                "fileNamePatterns": {
                    "type": "string",
                    "description": "Optional file name patterns (e.g., '*.java,*.xml'). If omitted, all files are searched."
                }
            },
            "required": ["containingText", "replacementText"]
        }
    },

    # ===== Run =====
    {
        "name": "eclipse_run",
        "description": "Run an Eclipse launch configuration.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "configuration": {
                    "type": "string",
                    "description": "Launch configuration name (optional - runs last used if omitted)"
                }
            },
            "required": []
        }
    },

    # ===== Refresh =====
    {
        "name": "eclipse_refresh",
        "description": "Refresh Eclipse workspace files or directories from disk.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Comma-separated paths to refresh"
                }
            },
            "required": ["path"]
        }
    },
    {
        "name": "eclipse_refresh_and_save",
        "description": "Refresh and save files, triggering Eclipse save actions (organize imports, format, etc.).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "paths": {
                    "type": "string",
                    "description": "Comma-separated paths to refresh and save"
                }
            },
            "required": ["paths"]
        }
    },

    # ===== Debug =====
    {
        "name": "eclipse_add_breakpoint",
        "description": "Add a breakpoint at a specific line in a file.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "lineNumber": {
                    "type": "integer",
                    "description": "Line number (1-based)"
                },
                "condition": {
                    "type": "string",
                    "description": "Optional conditional expression"
                }
            },
            "required": ["filePath", "lineNumber"]
        }
    },
    {
        "name": "eclipse_remove_breakpoint",
        "description": "Remove a breakpoint at a specific line.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "lineNumber": {
                    "type": "integer",
                    "description": "Line number"
                }
            },
            "required": ["filePath", "lineNumber"]
        }
    },
    {
        "name": "eclipse_list_breakpoints",
        "description": "List all breakpoints.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_enable_breakpoint",
        "description": "Enable a breakpoint at a specific line.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {"type": "string", "description": "Path to the file"},
                "lineNumber": {"type": "integer", "description": "Line number"}
            },
            "required": ["filePath", "lineNumber"]
        }
    },
    {
        "name": "eclipse_disable_breakpoint",
        "description": "Disable a breakpoint at a specific line.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filePath": {"type": "string", "description": "Path to the file"},
                "lineNumber": {"type": "integer", "description": "Line number"}
            },
            "required": ["filePath", "lineNumber"]
        }
    },
    {
        "name": "eclipse_list_threads",
        "description": "List all threads in the debug session.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_select_thread",
        "description": "Select a thread for inspection.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "threadId": {
                    "type": "string",
                    "description": "Thread name or ID"
                }
            },
            "required": ["threadId"]
        }
    },
    {
        "name": "eclipse_run_debug",
        "description": "Launch a debug configuration.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "configuration": {
                    "type": "string",
                    "description": "Debug configuration name"
                }
            },
            "required": ["configuration"]
        }
    },
    {
        "name": "eclipse_list_debug_configurations",
        "description": "List available launch configurations for run, debug, and coverage modes.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_create_java_launch_configuration",
        "description": "Create an Eclipse Java-based launch configuration.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "mainClass": {
                    "type": "string",
                    "description": "Fully qualified main or test class name"
                },
                "type": {
                    "type": "string",
                    "enum": ["java", "junit4", "junit5", "testng", "auto"],
                    "description": "Launch configuration type (server default when omitted)"
                },
                "configName": {
                    "type": "string",
                    "description": "Configuration name (derived from the main class when omitted)"
                },
                "projectName": {
                    "type": "string",
                    "description": "Project name (Eclipse may infer it when the main class is unique)"
                },
                "programArguments": {
                    "type": "string",
                    "description": "Arguments passed to the Java program"
                },
                "vmArguments": {
                    "type": "string",
                    "description": "Arguments passed to the JVM"
                },
                "workingDirectory": {
                    "type": "string",
                    "description": "Working directory (WSL paths are converted to Windows paths)"
                },
                "environmentVariables": {
                    "type": "object",
                    "additionalProperties": {"type": "string"},
                    "description": "Environment variable names and values"
                },
                "appendSystemEnvironment": {
                    "type": "boolean",
                    "description": "Inherit the system environment",
                    "default": True
                },
                "nameConflict": {
                    "type": "string",
                    "enum": ["error", "generate"],
                    "description": "Reject a duplicate name or generate a unique one",
                    "default": "error"
                }
            },
            "required": ["mainClass"]
        }
    },
    {
        "name": "eclipse_get_launch_configuration",
        "description": "Get launch configuration details. Environment values are masked unless explicitly requested.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "configName": {
                    "type": "string",
                    "description": "Configuration name"
                },
                "includeEnvironmentValues": {
                    "type": "boolean",
                    "description": "Return unmasked environment values",
                    "default": False
                }
            },
            "required": ["configName"]
        }
    },
    {
        "name": "eclipse_update_launch_environment",
        "description": "Update only the environment of an existing launch configuration.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "configName": {
                    "type": "string",
                    "description": "Configuration name"
                },
                "variables": {
                    "type": "object",
                    "additionalProperties": {"type": "string"},
                    "description": "Variables to merge or use as the complete replacement map"
                },
                "removeVariables": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Variable names to remove (not compatible with replace mode)"
                },
                "updateMode": {
                    "type": "string",
                    "enum": ["merge", "replace"],
                    "description": "Merge with or replace the configured variable map",
                    "default": "merge"
                },
                "appendSystemEnvironment": {
                    "type": "boolean",
                    "description": "Set system environment inheritance; omit to preserve the current value"
                }
            },
            "required": ["configName"]
        }
    },
    {
        "name": "eclipse_step_into",
        "description": "Step into the next method call (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_step_over",
        "description": "Step over the current line (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_step_return",
        "description": "Step out of the current method (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_debug_resume",
        "description": "Resume execution (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_debug_suspend",
        "description": "Suspend execution (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_debug_terminate",
        "description": "Terminate the debug session.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },
    {
        "name": "eclipse_get_stack_trace",
        "description": "Get the current stack trace (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "threadId": {
                    "type": "string",
                    "description": "Optional thread ID"
                }
            },
            "required": []
        }
    },
    {
        "name": "eclipse_get_variables",
        "description": "Get variables in the current scope (debug).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "frameId": {
                    "type": "string",
                    "description": "Optional stack frame ID"
                }
            },
            "required": []
        }
    },
    {
        "name": "eclipse_evaluate_expression",
        "description": "Evaluate an expression in the debug context.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Expression to evaluate"
                },
                "frameId": {
                    "type": "string",
                    "description": "Optional stack frame ID"
                }
            },
            "required": ["expression"]
        }
    },
    {
        "name": "eclipse_get_debug_status",
        "description": "Get current debug session status.",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    },

    # ===== Coverage =====
    {
        "name": "eclipse_run_coverage",
        "description": "Launch a configuration with code coverage analysis (EclEmma).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "configuration": {
                    "type": "string",
                    "description": "Launch configuration name"
                }
            },
            "required": ["configuration"]
        }
    },
    {
        "name": "eclipse_get_coverage_report",
        "description": "Get the code coverage report (line/branch/instruction coverage).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "projectName": {
                    "type": "string",
                    "description": "Optional project name to filter"
                }
            },
            "required": []
        }
    },
    {
        "name": "eclipse_get_class_coverage",
        "description": "Get coverage details for a specific class.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "className": {
                    "type": "string",
                    "description": "Fully qualified class name"
                }
            },
            "required": ["className"]
        }
    },

    # ===== Read Project Resource =====
    {
        "name": "eclipse_read_project_resource",
        "description": "Read a file from a project using a relative path.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "project": {
                    "type": "string",
                    "description": "Project name"
                },
                "path": {
                    "type": "string",
                    "description": "Relative path within the project (e.g., 'src/Main.java')"
                }
            },
            "required": ["project", "path"]
        }
    },
    {
        "name": "eclipse_import_project",
        "description": "Import a project into the Eclipse workspace.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the project directory"
                }
            },
            "required": ["path"]
        }
    },
]
