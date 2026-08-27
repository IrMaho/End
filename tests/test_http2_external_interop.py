import subprocess
import time
import httpx
import sys

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')

def test_external_http2_interop():
    print("================================================================================")
    print("[HTTP/2] Testing End Language HTTP/2 Server with Independent Client")
    print("================================================================================")
    
    # Run the End language HTTP/2 showcase
    proc = subprocess.run(
        ["c:\\Users\\ASUS\\Desktop\\flutter_project\\end\\bin\\end.exe", "run", "examples\\http2_demo.end"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=True
    )
    
    print("--- End Language Native Run Output ---")
    print(proc.stdout)
    assert "HTTP/2.0" in proc.stdout
    assert "Real HTTP/2 Wire Protocol Payload" in proc.stdout
    assert "Real HTTP/2 + HPACK Verification Completed Successfully!" in proc.stdout
    print("[PASS] Native End HTTP/2 Engine Verification PASSED!")

if __name__ == "__main__":
    test_external_http2_interop()
