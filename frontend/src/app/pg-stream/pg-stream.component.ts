import { ChangeDetectionStrategy, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { StreamEditComponent } from '../components/stream-edit/stream-edit.component';

@Component({
  selector: 'app-pg-stream',
  standalone: true,
  imports: [CommonModule, RouterOutlet, StreamEditComponent],
  templateUrl: './pg-stream.component.html',
  styleUrls: ['./pg-stream.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamComponent implements OnInit {
  constructor() {
    console.log(`PgStreamComponent()`); // #-
  }
  ngOnInit(): void {
    console.log(`PgStreamComponent().OnInit()`); // #-
  }
}
  
