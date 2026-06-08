import LottieView from 'lottie-react-native';

export function SuccessAnimation() {
  return <LottieView source={require('./success.json')} autoPlay loop={false} accessibilityLabel="Success" />;
}
