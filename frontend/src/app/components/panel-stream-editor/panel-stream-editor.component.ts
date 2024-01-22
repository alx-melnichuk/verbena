import { ChangeDetectionStrategy, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-panel-stream-editor',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './panel-stream-editor.component.html',
  styleUrls: ['./panel-stream-editor.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamEditorComponent implements OnInit {
    constructor() {
      console.log(`PanelStreamEditorComponent()`); // #-
    }
    ngOnInit(): void {
      console.log(`PanelStreamEditorComponent().OnInit()`); // #-
    }
  }
