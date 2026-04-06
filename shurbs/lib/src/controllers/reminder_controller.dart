import 'dart:convert';

import 'package:flutter/foundation.dart';

import '../models/reminder_model.dart';
import '../rust/api/reminders.dart';

class ReminderController extends ChangeNotifier {
  List<Reminder> _reminders = [];
  bool loading = true;
  String? _workspaceId;

  List<Reminder> get reminders => List.unmodifiable(_reminders);

  Future<void> load(String workspaceId) async {
    _workspaceId = workspaceId;
    try {
      final raw = await getAllReminders(metaWorkspaceId: workspaceId);
      final list = (jsonDecode(raw) as List).cast<Map<String, dynamic>>();
      _reminders = list.map(Reminder.fromJson).toList();
    } catch (e) {
      debugPrint('ReminderController.load error: $e');
    }
    loading = false;
    notifyListeners();
  }

  Future<void> create({
    required String title,
    required String remindAt,
    required bool recurring,
    String? recurrenceRule,
  }) async {
    if (_workspaceId == null) return;
    try {
      final raw = await createReminder(
        title: title,
        remindAt: remindAt,
        recurring: recurring,
        recurrenceRule: recurrenceRule,
        metaWorkspaceId: _workspaceId,
      );
      final json = jsonDecode(raw) as Map<String, dynamic>;
      _reminders.add(Reminder.fromJson(json));
      notifyListeners();
    } catch (e) {
      debugPrint('ReminderController.create error: $e');
    }
  }

  Future<void> toggleRecurring(Reminder reminder) async {
    if (_workspaceId == null) return;
    try {
      final next = !reminder.recurring;
      await updateReminder(
        identifier: reminder.id,
        recurring: next,
        metaWorkspaceId: _workspaceId,
      );
      reminder.recurring = next;
      notifyListeners();
    } catch (e) {
      debugPrint('ReminderController.toggleRecurring error: $e');
    }
  }

  Future<void> delete(Reminder reminder) async {
    if (_workspaceId == null) return;
    try {
      await deleteReminder(identifier: reminder.id, metaWorkspaceId: _workspaceId);
      _reminders.removeWhere((r) => r.id == reminder.id);
      notifyListeners();
    } catch (e) {
      debugPrint('ReminderController.delete error: $e');
    }
  }
}
