import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamEditor } from './panel-stream-editor';

describe('PanelStreamEditor', () => {
  let component: PanelStreamEditor;
  let fixture: ComponentFixture<PanelStreamEditor>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamEditor]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamEditor);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
