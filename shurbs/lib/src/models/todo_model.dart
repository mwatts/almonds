class Todo {
  final String id;
  String title;
  String priority;
  bool completed;

  Todo({
    required this.id,
    required this.title,
    required this.priority,
    this.completed = false,
  });

  factory Todo.fromJson(Map<String, dynamic> j) => Todo(
        id: j['identifier'] as String,
        title: j['title'] as String,
        priority: j['priority'] as String? ?? 'medium',
        completed: j['done'] as bool? ?? false,
      );
}
