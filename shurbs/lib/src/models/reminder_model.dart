import 'package:flutter/material.dart';

class Reminder {
  final String id;
  String title;
  String? description;
  DateTime remindAt;
  bool recurring;
  String? recurrenceRule;

  Reminder({
    required this.id,
    required this.title,
    this.description,
    required this.remindAt,
    this.recurring = false,
    this.recurrenceRule,
  });

  factory Reminder.fromJson(Map<String, dynamic> j) => Reminder(
        id: j['identifier'] as String,
        title: j['title'] as String,
        description: j['description'] as String?,
        remindAt: DateTime.parse(j['remindAt'] as String),
        recurring: j['recurring'] as bool? ?? false,
        recurrenceRule: j['recurrenceRule'] as String?,
      );

  TimeOfDay get time => TimeOfDay(hour: remindAt.hour, minute: remindAt.minute);

  List<String> get days {
    if (recurrenceRule == null || recurrenceRule!.isEmpty) return [];
    return recurrenceRule!.split(',').where((d) => d.isNotEmpty).toList();
  }
}
