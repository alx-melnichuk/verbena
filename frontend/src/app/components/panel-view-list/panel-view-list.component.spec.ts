import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelViewListComponent } from './panel-view-list.component';

describe('PanelViewListComponent', () => {
  let component: PanelViewListComponent;
  let fixture: ComponentFixture<PanelViewListComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelViewListComponent]
    });
    fixture = TestBed.createComponent(PanelViewListComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
