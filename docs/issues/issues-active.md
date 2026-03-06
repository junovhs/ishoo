# ACTIVE Issues

---
[NEW] Displaced cards should be locked in place before release
User's description:
Cards occupy fixed slots in a deterministic grid. Each slot has a fixed height. When you drag a card and others move out of the way, that movement happens during the drag — those cards animate to their new grid positions while you are still holding. By the time you release, the displaced cards are already sitting exactly where they belong. They are done. They do not move on release. Only the dragged card itself needs to animate into its final slot on release.
Current broken behaviour:
Displaced cards animate or jump on release, causing visible gaps and overlaps. This is wrong because those cards should already be in their final positions before the user lets go.
Desired behaviour:

During drag: displaced cards move to their offset grid positions (this is already working)
On release: displaced cards do not move at all — they are already correct
On release: only the dragged card animates into its new slot

user is extremely frustrated with the current ordering system being fundamentally broken.
