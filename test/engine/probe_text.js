(function(){
function report(){
  try{
    var w=document.createTreeWalker(document.body,NodeFilter.SHOW_TEXT),n,hits=[];
    while((n=w.nextNode())&&hits.length<3){ if(/something went wrong/i.test(n.nodeValue)){ var e=n.parentNode,chain=[]; for(var d=0;d<6&&e&&e!==document.body;d++){ var a=[]; for(var i=0;i<e.attributes.length;i++){a.push(e.attributes[i].name+'='+String(e.attributes[i].value).slice(0,40))} var cs=getComputedStyle(e); chain.push(e.tagName+'['+a.join(' ')+'] display='+cs.display+' vis='+cs.visibility); e=e.parentNode } hits.push(chain) } }
    var pops=document.querySelectorAll('[popover]').length, dialogs=document.querySelectorAll('dialog:not([open])').length, dialogsVisible=Array.prototype.filter.call(document.querySelectorAll('dialog:not([open])'),function(d){return getComputedStyle(d).display!=='none'}).length;
    document.title='AMNIPROBE '+JSON.stringify({hits:hits,popover:pops,closedDialogs:dialogs,closedDialogsVisible:dialogsVisible});
  }catch(e){document.title='AMNIPROBE-ERR '+e}
}
setTimeout(report,6000);
return 'scheduled';
})()
