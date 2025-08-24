pipeline {
    agent {
        docker { image 'rust:latest' }
    }
    stages {
        stage('Setup Rust') {
            steps {
                bat '''
                    curl -sSf -o rustup-init.exe https://win.rustup.rs/x86_64
                    rustup-init.exe -y
                    set PATH=%USERPROFILE%\\.cargo\\bin;%PATH%
                    rustc --version
                    cargo --version
                '''
            }
        }

        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Build') {
            steps {
                bat '''
                    set PATH=%USERPROFILE%\\.cargo\\bin;%PATH%
                    cargo build --verbose
                '''
            }
        }

        stage('Integration Tests') {
            steps {
                bat '''
                    set PATH=%USERPROFILE%\\.cargo\\bin;%PATH%
                    cargo test --test integration_udp --verbose
                '''
            }
        }

        stage('Stress Tests') {
            steps {
                bat '''
                    set PATH=%USERPROFILE%\\.cargo\\bin;%PATH%
                    cargo test --test stress_test --verbose
                '''
            }
        }

        stage('Package') {
            steps {
                bat '''
                    set PATH=%USERPROFILE%\\.cargo\\bin;%PATH%
                    cargo build --release
                '''
                archiveArtifacts artifacts: 'target/release/*', fingerprint: true
            }
        }
    }

    post {
        success {
            echo '✅ Build, Integration & Stress Tests Passed!'
        }
        failure {
            echo '❌ Build or Tests Failed.'
        }
    }
}
