Changed

- **A spend checkpoint keeps page images in the generation file.**
  The live directory stores where each page sits in that file and reads it
  on demand, including an explicit zero. Fold copies those file bytes onto
  `spent.body`. The open set also asks for a checkpoint at 65,536 distinct
  pages and keeps asking until that set drains. Freeze wakes a confirm
  thread blocked on the edge cap before the cooled-page copy.
