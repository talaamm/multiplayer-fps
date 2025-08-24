pipeline {
    agent any

    stages {
        stage('Setup Rust') {
            steps {
                sh '''
                    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                    export PATH="$HOME/.cargo/bin:$PATH"
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
                sh '''
                    export PATH="$HOME/.cargo/bin:$PATH"
                    cargo build --verbose
                '''
            }
        }

        stage('Test') {
            steps {
                sh '''
                    export PATH="$HOME/.cargo/bin:$PATH"
                    cargo test --verbose
                '''
            }
        }

        stage('Package') {
            steps {
                sh '''
                    export PATH="$HOME/.cargo/bin:$PATH"
                    cargo build --release
                '''
                archiveArtifacts artifacts: 'target/release/*', fingerprint: true
            }
        }
    }

    post {
        success {
            echo '✅ Build & Tests Passed!'
        }
        failure {
            echo '❌ Build or Tests Failed.'
        }
    }
}
