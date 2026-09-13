import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelBrowseState } from './panel-browse-state';

describe('PanelBrowseState', () => {
  let component: PanelBrowseState;
  let fixture: ComponentFixture<PanelBrowseState>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelBrowseState]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelBrowseState);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
