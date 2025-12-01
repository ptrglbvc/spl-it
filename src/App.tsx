import { useState, useRef, useEffect, ChangeEvent, UIEvent } from "react";
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import "./App.css";

function App() {
    const [code, setCode] = useState<string>("println(\"Wake up.\")\nThread.sleep(1000)\nprintln(\"Time to die.\")");
    const [codeOutput, setCodeOutput] = useState<string>("");
    const [isLoading, setIsLoading] = useState<boolean>(false);
    
    // Refs for scrolling
    const backdropRef = useRef<HTMLDivElement>(null);
    const outputRef = useRef<HTMLDivElement>(null);

    // Sync editor scroll
    const handleScroll = (e: UIEvent<HTMLTextAreaElement>) => {
        if (backdropRef.current) {
            const target = e.currentTarget; 
            backdropRef.current.scrollTop = target.scrollTop;
            backdropRef.current.scrollLeft = target.scrollLeft;
        }
    };

    // Auto-scroll output to bottom when text changes
    useEffect(() => {
        if (outputRef.current) {
            outputRef.current.scrollTop = outputRef.current.scrollHeight;
        }
    }, [codeOutput]);

    const handleChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
        setCode(e.target.value);
    };

    const handleCodeSubmit = async () => {
        if (isLoading) return;
        
        setIsLoading(true);
        setCodeOutput(""); 
        
        const unlisten = await listen<string>('stream-data', (event) => {
            setCodeOutput((prev) => prev + event.payload);
        });
        
        try {
            await invoke("run_kotlin_code", { code });
        } catch (error) {
            console.error(error);
            setCodeOutput((prev) => prev + `\nSystem Error: ${error}`);
        } finally {
            unlisten(); // Stop listening, show's over
            setIsLoading(false);
        }
    }

    const highlightCode = (text: string): React.ReactNode[] => {
        const regex = /("(?:[^"\\]|\\.)*"|\/\/.*$|\b(?:fun|val|var|class|when|if|else|return|true|false|null)\b|\b\d+\b)/gm;

        return text.split(regex).map((part, index) => {
            if (part.startsWith('"')) {
                return <span key={index} className="token-string">{part}</span>;
            } else if (part.startsWith('//')) {
                return <span key={index} className="token-comment">{part}</span>;
            } else if (/^\d+$/.test(part)) {
                return <span key={index} className="token-number">{part}</span>;
            } else if (/^(fun|val|var|class|when|if|else|return|true|false|null)$/.test(part)) {
                return <span key={index} className="token-keyword">{part}</span>;
            } else {
                return part;
            }
        });
    };

    return (
        <main className="container">
            <div className="code-editor">
                <div 
                    className="syntax-layer" 
                    aria-hidden="true"
                    ref={backdropRef} 
                >
                    {highlightCode(code)}
                    <br /> 
                </div>
                <textarea
                    value={code}
                    onChange={handleChange}
                    onScroll={handleScroll} 
                    className="input-layer"
                    spellCheck="false"
                />
                <button 
                    className={`submit ${isLoading ? 'loading' : ''}`} 
                    onClick={handleCodeSubmit}
                    disabled={isLoading}
                >
                    {isLoading ? 'Running...' : 'Run'}
                </button>
            </div>
            <div className="code-output">
                <div className="output-label">Console</div>
                <div className="code-output-console" ref={outputRef}>
                    {isLoading && codeOutput === "" ? <span className="blinking-cursor">_</span> : codeOutput}
                </div>
            </div>
        </main>
    );
}

export default App;
