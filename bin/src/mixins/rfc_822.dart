mixin Rfc822 {
  static const Map<String, String> _months = {
    'Jan': '01',
    'Feb': '02',
    'Mar': '03',
    'Apr': '04',
    'May': '05',
    'Jun': '06',
    'Jul': '07',
    'Aug': '08',
    'Sep': '09',
    'Oct': '10',
    'Nov': '11',
    'Dec': '12',
  };

  static DateTime? parse(String input) {
    if (input.isEmpty) return null;

    final List<String> splits = input
        .replaceFirst('GMT', '+0000')
        .replaceFirst('UTC', '+0000')
        .split(' ');

    final String year = splits[3];

    final String? month = _months[splits[2]];
    if (month == null) return null;

    String day = splits[1];
    if (day.length == 1) day = '0$day';

    final String splitTime = splits[4], splitZone = splits[5];

    return DateTime.tryParse(
      '$year-$month-$day $splitTime $splitZone',
    );
  }
}
