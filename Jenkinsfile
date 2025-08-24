pipeline {
    agent any

    stages {
        stage('Checkout') {
            steps {
                checkout scm
            }
        }

        stage('Build & Tests in Docker') {
            steps {
                script {
                    docker.image('rust:latest').inside {
                        bat 'rustc --version || echo ok'
                        bat 'cargo --version || echo ok'

                        
                        bat 'cargo build --verbose'

                       
                        bat 'cargo test --test integration_udp --verbose'

                    }
                }
            }
        }

        stage('Package') {
            steps {
                script {
                    docker.image('rust:latest').inside {
                        bat 'cargo build --release'
                        archiveArtifacts artifacts: 'target/release/*', fingerprint: true
                    }
                }
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
