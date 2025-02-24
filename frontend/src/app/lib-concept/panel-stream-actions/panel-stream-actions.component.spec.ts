import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamActionsComponent } from './panel-stream-actions.component';

describe('PanelStreamActionsComponent', () => {
  let component: PanelStreamActionsComponent;
  let fixture: ComponentFixture<PanelStreamActionsComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamActionsComponent]
    });
    fixture = TestBed.createComponent(PanelStreamActionsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
