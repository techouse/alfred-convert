import 'package:envied/envied.dart';

part 'env.g.dart';

@Envied(path: '.env')
abstract class Env {
  @EnviedField(varName: 'APP_VERSION')
  static const String appVersion = _Env.appVersion;

  @EnviedField(varName: 'GITHUB_REPOSITORY_URL')
  static const String githubRepositoryUrl = _Env.githubRepositoryUrl;
}
