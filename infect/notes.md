
Steps:
1. File Discovery/Targeting
    - recursive scanning
2. Encryption
    - use asymmetric keys
3. Communicate with C2 Server
    - build server in Python
4. Exfiltration
    - send files/data to C2 Server
5. Persistence
    - Windows
    - Linux
6. Hide Tracks/Stealth
7. Evasion Techniques
8. Lateral Movement


Components

- Logging
    - Files found
        - log every target file, skipped file
    - Encryption 
        - log creation of keys, encryption of files (success/failure)
    - Decryption
        - log decryption of files (success/failure)
    - Ransom note generation log
    - Data extraction
        - log files sent to C2 Server
    - Persistence
        - log persistence mechanism (success/failure)
    - include timestamp and computer info
    - needs to send log over network

- File Encryption
    - Symmetric: Fast and efficient but uses a single key for encryption and decryption (e.g., AES). This requires the ransomware to somehow store or send the key back to the attacker (or C2 server)
    - Asymmetric: The ransomware encrypts files using the public key (included in the malware), and only the attacker has the private key to decrypt the files
    - Hybrid: Many ransomware variants use hybrid encryption. For example, each file may be encrypted with a symmetric AES key, and then that AES key is encrypted using an RSA public key. This approach combines speed (symmetric encryption) with the security of asymmetric encryption

- C2 Infrastructure -> python and mysql docker containers 
    - SQL database with:
        - Encryption Key Management
            - figure out how to send/store public/private key
        - Payment Handling (paid or not with amount)
        - Victims Identification
            - hostname, unique UUID, os, IP
        - make sure database for mysql is mounted somewhere so if docker crashes all data still available
    - Data Exfiltration

- File Discovery and Targeting
    - Directory Traversal: The malware recursively scans the file system, looking for files with specific extensions (e.g., .docx, .xls, .pdf) to encrypt
    - File Extensions and Blacklists: The ransomware often targets specific file extensions while avoiding system-critical files (like .dll, .exe, or OS-specific files) to avoid crashing the system before completing the encryption process

- Ransom Note Generation

- Persistence
    - build a back door
    - Windows: Ransomware often modifies the registry (e.g., HKCU\Software\Microsoft\Windows\CurrentVersion\Run) or schedules a task to restart itself.
    - Linux/mac: Ransomware can set up cron jobs (Linux) or use launch daemons (macOS) to achieve persistence.

- Lateral Movement:
    - Credential Harvesting
    - Network Scanning
    - SMB/HTTP/FTP Exploitation

- Anti-analysis and Evasion Techniques
    - Code Obfuscation
    - Anti-debugging
    - Anti-emulation
    - Anti-sandboxing
    - Packing

- Hide Evidence of Malware

- Create TOML config file that stores
    - C2 IP info
    - file extension
    - other

- Deployment
    - cargo build --release
    - have one executable for encrypt and one for decrypt (windows and linux, 4 total)
        - decrypt should have decryption key in executable
        - decrypt also does not need rest of rust ransom code