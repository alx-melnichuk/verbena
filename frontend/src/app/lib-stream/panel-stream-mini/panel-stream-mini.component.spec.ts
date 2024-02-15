import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamMiniComponent } from './panel-stream-mini.component';

describe('PanelStreamMiniComponent', () => {
  let component: PanelStreamMiniComponent;
  let fixture: ComponentFixture<PanelStreamMiniComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamMiniComponent]
    });
    fixture = TestBed.createComponent(PanelStreamMiniComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
