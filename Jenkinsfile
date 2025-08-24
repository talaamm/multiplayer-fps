pipeline {
    agent any

    tools {
        rust 'rust-stable'
    }

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Build') {
            steps {
                sh 'cargo build --verbose'
            }
        }

        stage('Test') {
            steps {
                sh 'cargo test --verbose'
            }
        }

        stage('Package') {
            steps {
                sh 'cargo build --release'
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